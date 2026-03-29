use html_to_markdown_rs::{convert, conversion_options_from_json, ConversionOptions};
use lol_html::{element, rewrite_str, RewriteStrSettings};
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod data_ai;

pub fn prune_data_ai_ignore(html: &str) -> Result<String, String> {
    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("*[data-ai-ignore]", |el| {
                el.remove();
                Ok(())
            })],
            ..RewriteStrSettings::new()
        },
    )
    .map_err(|e| e.to_string())
}

/// Allowed `<meta name="…">` values for agent-facing YAML frontmatter (ASCII case-insensitive).
/// Twitter Card tags use `name="twitter:…"`, covered by [`meta_name_allowed`].
const AGENT_ALLOWED_META_NAMES: &[&str] = &[
    "author",
    "description",
    "keywords",
    "publisher",
    "robots",
];

fn meta_name_allowed(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    AGENT_ALLOWED_META_NAMES
        .iter()
        .any(|allowed| lower == *allowed)
        || lower.starts_with("twitter:")
        || lower.starts_with("dc.")
        || lower.starts_with("dc-")
        || lower.starts_with("dcterms.")
        || lower.starts_with("dcterms-")
}

fn meta_property_allowed(property: &str) -> bool {
    let lower = property.to_ascii_lowercase();
    lower.starts_with("og:")
        || lower.starts_with("article:")
        || lower.starts_with("book:")
        || lower.starts_with("music:")
}

fn meta_allowed_for_agents(name: Option<&str>, property: Option<&str>) -> bool {
    match (name, property) {
        (Some(n), None) => meta_name_allowed(n),
        (None, Some(p)) => meta_property_allowed(p),
        (Some(n), Some(p)) => meta_name_allowed(n) || meta_property_allowed(p),
        (None, None) => true,
    }
}

/// Drop `<meta>` tags that are not on the agent allowlist or have no non-empty `content`.
/// `charset` / other metas without `name` and `property` are left unchanged.
pub fn prune_agent_useless_head_metas(html: &str) -> Result<String, String> {
    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("meta", |el| {
                let name = el.get_attribute("name");
                let property = el.get_attribute("property");
                if name.is_none() && property.is_none() {
                    return Ok(());
                }
                let content = el.get_attribute("content");
                let content_nonempty = content
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .is_some();
                if !content_nonempty
                    || !meta_allowed_for_agents(name.as_deref(), property.as_deref())
                {
                    el.remove();
                }
                Ok(())
            })],
            ..RewriteStrSettings::new()
        },
    )
    .map_err(|e| e.to_string())
}

fn parse_conversion_options(json: Option<&str>) -> Result<Option<ConversionOptions>, String> {
    match json {
        None | Some("") => Ok(None),
        Some(raw) => conversion_options_from_json(raw).map(Some).map_err(|e| e.to_string()),
    }
}

pub fn html_to_markdown_inner(html: &str, options_json: Option<&str>) -> Result<String, String> {
    let opts = parse_conversion_options(options_json)?;
    convert(html, opts).map_err(|e| e.to_string())
}

/// Returns `(markdown, token_count, truncated)`.
/// With `tiktoken` feature: uses cl100k_base. Without (e.g. WASM): approximate ~4 chars per token.
pub fn apply_token_limit(
    markdown: &str,
    max_tokens: Option<u32>,
    continued: &str,
) -> Result<(String, u32, bool), String> {
    #[cfg(feature = "tiktoken")]
    {
        let Some(max) = max_tokens.filter(|m| *m > 0) else {
            let bpe = tiktoken_rs::cl100k_base().map_err(|e| e.to_string())?;
            let tokens = bpe.encode_with_special_tokens(markdown);
            return Ok((markdown.to_string(), tokens.len() as u32, false));
        };

        let max = max as usize;
        let bpe = tiktoken_rs::cl100k_base().map_err(|e| e.to_string())?;
        let tokens = bpe.encode_with_special_tokens(markdown);
        let count = tokens.len() as u32;

        if tokens.len() <= max {
            return Ok((markdown.to_string(), count, false));
        }

        let slice = &tokens[..max];
        let mut out = bpe
            .decode(slice.to_vec())
            .map_err(|e| format!("tiktoken decode: {e}"))?;
        out.push_str("\n\n");
        out.push_str(continued);
        out.push('\n');

        return Ok((out, max as u32, true));
    }

    #[cfg(not(feature = "tiktoken"))]
    {
        let approx_count = ((markdown.len() as f64) / 4.0).ceil() as u32;
        let Some(max) = max_tokens.filter(|m| *m > 0) else {
            return Ok((markdown.to_string(), approx_count, false));
        };

        let max_chars = (max as usize).saturating_mul(4);
        if markdown.len() <= max_chars {
            return Ok((markdown.to_string(), approx_count, false));
        }

        let mut cut = max_chars.min(markdown.len());
        while cut > 0 && !markdown.is_char_boundary(cut) {
            cut -= 1;
        }
        let mut out = markdown[..cut].to_string();
        out.push_str("\n\n");
        out.push_str(continued);
        out.push('\n');
        Ok((out, max, true))
    }
}

pub fn convert_pipeline(
    html: &str,
    conversion_options_json: Option<&str>,
    max_tokens: Option<u32>,
    continued_message: Option<&str>,
) -> Result<(String, u32, bool), String> {
    let pruned = prune_data_ai_ignore(html)?;
    let (pruned, ai_meta) = data_ai::apply_data_ai_html_transforms(&pruned)?;
    let pruned = prune_agent_useless_head_metas(&pruned)?;
    let md = html_to_markdown_inner(&pruned, conversion_options_json)?;
    let md = data_ai::merge_ai_meta_into_markdown(md, &ai_meta);
    let continued = continued_message.unwrap_or("[CONTINUED IN NEXT CHUNK]");
    apply_token_limit(&md, max_tokens, continued)
}

pub fn convert_file_parallel(
    input_root: &Path,
    output_root: &Path,
    conversion_options_json: Option<&str>,
    max_tokens: Option<u32>,
    continued_message: Option<&str>,
) -> Result<usize, String> {
    let opts_owned = conversion_options_json.map(|s| s.to_string());
    let continued_owned = continued_message.map(|s| s.to_string());

    let entries: Vec<PathBuf> = WalkDir::new(input_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect();

    let input_root = input_root.to_path_buf();
    let output_root = output_root.to_path_buf();

    let written = entries
        .par_iter()
        .filter_map(|path| {
            let html = fs::read_to_string(path).ok()?;
            let (md, _, _) = convert_pipeline(
                &html,
                opts_owned.as_deref(),
                max_tokens,
                continued_owned.as_deref(),
            )
            .ok()?;
            let rel = path.strip_prefix(&input_root).unwrap_or(path);
            let out_path = output_root.join(rel).with_extension("md");
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok()?;
            }
            let mut f = fs::File::create(&out_path).ok()?;
            f.write_all(md.as_bytes()).ok()?;
            Some(())
        })
        .count();

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn prune_removes_elements_with_data_ai_ignore() {
        let html = r#"<div><p id="keep">Visible</p><aside data-ai-ignore><b>Hidden</b></aside></div>"#;
        let out = prune_data_ai_ignore(html).unwrap();
        assert!(out.contains("Visible"));
        assert!(!out.contains("Hidden"));
        assert!(!out.contains("data-ai-ignore"));
    }

    #[test]
    fn prune_preserves_html_without_ignore() {
        let html = r#"<p>All <em>content</em> stays.</p>"#;
        let out = prune_data_ai_ignore(html).unwrap();
        assert!(out.contains("content"));
    }

    #[test]
    fn html_to_markdown_simple_heading() {
        let md = html_to_markdown_inner("<h1>Hello</h1><p>World.</p>", None).unwrap();
        assert!(md.contains("Hello"));
        assert!(md.contains("World"));
    }

    #[test]
    fn html_to_markdown_invalid_options_json_errors() {
        let err = html_to_markdown_inner("<p>x</p>", Some("not json")).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn convert_pipeline_prunes_then_converts() {
        let html = r#"<main><h2>Docs</h2><div data-ai-ignore>noise</div><p>Body.</p></main>"#;
        let (md, _tokens, truncated) =
            convert_pipeline(html, None, None, None).expect("pipeline");
        assert!(md.contains("Docs"));
        assert!(md.contains("Body"));
        assert!(!md.contains("noise"));
        assert!(!truncated);
    }

    #[test]
    fn prune_agent_head_metas_keeps_allowlist_only() {
        let html = r#"<head>
<meta charset="utf-8">
<meta name="description" content="Keep me">
<meta name="viewport" content="width=device-width">
<meta name="next-size-adjust" content="">
<meta name="google-site-verification" content="token">
</head><body><p>x</p></body>"#;
        let out = prune_agent_useless_head_metas(html).unwrap();
        assert!(out.contains("description"));
        assert!(out.contains("Keep me"));
        assert!(!out.contains("viewport"));
        assert!(!out.contains("next-size-adjust"));
        assert!(!out.contains("google-site-verification"));
        assert!(out.contains(r#"charset="utf-8""#));
    }

    #[test]
    fn prune_agent_head_metas_keeps_og_and_twitter() {
        let html = r#"<head>
<meta property="og:title" content="OG Title">
<meta name="twitter:card" content="summary_large_image">
<meta name="generator" content="Next.js">
</head><body></body>"#;
        let out = prune_agent_useless_head_metas(html).unwrap();
        assert!(out.contains("og:title"));
        assert!(out.contains("twitter:card"));
        assert!(!out.contains("generator"));
    }

    #[test]
    fn convert_pipeline_frontmatter_omits_useless_meta() {
        let html = r#"<!DOCTYPE html><html><head>
<meta name="description" content="For agents">
<meta name="next-size-adjust" content="">
<meta name="viewport" content="width=device-width">
<title>Example</title>
</head><body><h1>Hi</h1></body></html>"#;
        let (md, _, _) = convert_pipeline(html, None, None, None).expect("pipeline");
        assert!(md.contains("meta-description:"));
        assert!(md.contains("For agents"));
        assert!(!md.contains("meta-next-size-adjust"));
        assert!(!md.contains("meta-viewport"));
    }

    #[test]
    fn apply_token_limit_none_returns_full_markdown() {
        let text = "# Short\n\nParagraph.";
        let (out, _count, truncated) = apply_token_limit(text, None, "[END]").unwrap();
        assert_eq!(out, text);
        assert!(!truncated);
    }

    #[test]
    fn apply_token_limit_truncates_when_over_max() {
        let repeated = "word ".repeat(5000);
        let unique_suffix = "<<<UNIQUE_TAIL_ONLY_HERE>>>";
        let body = format!("{repeated}{unique_suffix}");
        let continued = "[CONTINUED]";
        let (out, _count, truncated) = apply_token_limit(&body, Some(8), continued).expect("limit");
        assert!(truncated);
        assert!(out.len() < body.len());
        assert!(out.contains(continued));
        assert!(
            !out.contains(unique_suffix),
            "suffix should be dropped when truncating to a small token budget"
        );
    }

    #[test]
    fn convert_file_parallel_writes_markdown() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("sub");
        fs::create_dir_all(&input).unwrap();
        let html_path = input.join("page.html");
        let mut f = fs::File::create(&html_path).unwrap();
        write!(
            f,
            r#"<html><body><h1>File</h1><p data-ai-ignore>x</p><p>OK</p></body></html>"#
        )
        .unwrap();

        let out_dir = tmp.path().join("out");
        let n = convert_file_parallel(
            &input,
            &out_dir,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(n, 1);

        // Paths mirror `strip_prefix(input_root)` → `page.html` becomes `out/page.md`.
        let md_path = out_dir.join("page.md");
        let md = fs::read_to_string(&md_path).unwrap();
        assert!(md.contains("File"));
        assert!(md.contains("OK"));
        assert!(!md.contains('x'));
    }
}

#[cfg(all(test, not(feature = "tiktoken")))]
mod tests_no_tiktoken {
    use super::*;

    #[test]
    fn apply_token_limit_approx_truncation_without_tiktoken() {
        let body = "a".repeat(100);
        let (out, count, truncated) = apply_token_limit(&body, Some(5), "[MORE]").unwrap();
        assert!(truncated);
        assert_eq!(count, 5);
        assert!(out.ends_with("[MORE]\n"));
        assert!(out.len() < body.len() + 20);
    }
}
