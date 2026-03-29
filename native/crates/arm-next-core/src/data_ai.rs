//! `data-ai-meta` HTML directive for agent-oriented Markdown output (YAML frontmatter).

use lol_html::html_content::EndTag;
use lol_html::{element, rewrite_str, text, EndTagHandler, HandlerResult, RewriteStrSettings};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Keys must be safe YAML identifiers.
fn is_valid_ai_meta_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// True if the value cannot be written as a one-line YAML plain scalar after `key: `.
fn yaml_plain_scalar_needs_quotes(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    if value.contains('\n') || value.contains('\r') {
        return true;
    }
    if value.starts_with(' ') || value.ends_with(' ') {
        return true;
    }
    // `key: foo: bar` — colon-space inside the value is ambiguous with nested mappings.
    if value.contains(": ") || value.starts_with(':') {
        return true;
    }
    if value.contains('#') {
        return true;
    }
    if value.contains('"') || value.contains('\'') {
        return true;
    }
    if value.chars().any(|c| c == '{' || c == '}' || c == '[' || c == ']') {
        return true;
    }
    // Avoid YAML 1.1 bool/null coercion for string metadata.
    let lower = value.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "true" | "false" | "yes" | "no" | "on" | "off" | "null" | "~"
    ) {
        return true;
    }
    false
}

fn quote_yaml_scalar(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    if !yaml_plain_scalar_needs_quotes(value) {
        return value.to_string();
    }
    let mut s = String::with_capacity(value.len() + 2);
    s.push('"');
    for c in value.chars() {
        match c {
            '\\' => s.push_str("\\\\"),
            '"' => s.push_str("\\\""),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            _ => s.push(c),
        }
    }
    s.push('"');
    s
}

/// Collect `[data-ai-meta="key"]` text (normalized whitespace). Nested `data-ai-meta` is not supported.
pub fn collect_data_ai_meta(html: &str) -> Result<Vec<(String, String)>, String> {
    let metas: Rc<RefCell<BTreeMap<String, String>>> = Rc::new(RefCell::new(BTreeMap::new()));
    let text_buf: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    let metas_el = metas.clone();
    let text_buf_el = text_buf.clone();

    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![
                element!("*[data-ai-meta]", {
                    let metas_el = metas_el.clone();
                    let text_buf_el = text_buf_el.clone();
                    move |el| {
                        let Some(key_raw) = el.get_attribute("data-ai-meta") else {
                            return Ok(());
                        };
                        let key = key_raw.trim();
                        if !is_valid_ai_meta_key(key) {
                            return Ok(());
                        }
                        let key = key.to_string();
                        text_buf_el.borrow_mut().clear();
                        let tb = text_buf_el.clone();
                        let ms = metas_el.clone();
                        if let Some(h) = el.end_tag_handlers() {
                            let key2 = key.clone();
                            let end_cb: EndTagHandler<'static> =
                                Box::new(move |_end: &mut EndTag<'_>| -> HandlerResult {
                                    let t = tb.borrow();
                                    let normalized: String =
                                        t.split_whitespace().collect::<Vec<_>>().join(" ");
                                    if !normalized.is_empty() {
                                        ms.borrow_mut().insert(key2.clone(), normalized);
                                    }
                                    Ok(())
                                });
                            h.push(end_cb);
                        }
                        Ok(())
                    }
                }),
                text!("*[data-ai-meta]", {
                    let text_buf = text_buf.clone();
                    move |t| {
                        text_buf.borrow_mut().push_str(t.as_str());
                        Ok(())
                    }
                }),
            ],
            ..RewriteStrSettings::new()
        },
    )
    .map_err(|e| e.to_string())?;

    let pairs: Vec<(String, String)> = metas.borrow().clone().into_iter().collect();
    Ok(pairs)
}

pub fn strip_data_ai_meta_attributes(html: &str) -> Result<String, String> {
    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("*[data-ai-meta]", |el| {
                el.remove_attribute("data-ai-meta");
                Ok(())
            })],
            ..RewriteStrSettings::new()
        },
    )
    .map_err(|e| e.to_string())
}

pub fn apply_data_ai_html_transforms(html: &str) -> Result<(String, Vec<(String, String)>), String> {
    let meta = collect_data_ai_meta(html)?;
    let html = strip_data_ai_meta_attributes(html)?;
    Ok((html, meta))
}

fn parse_simple_frontmatter_body(md: &str) -> Option<(&str, &str, &str)> {
    if !md.starts_with("---\n") {
        return None;
    }
    let rest = md.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    let inner = &rest[..end];
    let after = &rest[end + 5..];
    Some(("---\n", inner, after))
}

fn parse_frontmatter_map(inner: &str) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for line in inner.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim().to_string();
            let v = v.trim().to_string();
            if !k.is_empty() {
                m.insert(k, v);
            }
        }
    }
    m
}

fn format_frontmatter(map: &BTreeMap<String, String>) -> String {
    let mut s = String::from("---\n");
    for (k, v) in map {
        use std::fmt::Write as _;
        let _ = writeln!(&mut s, "{}: {}", k, v);
    }
    s.push_str("---\n");
    s
}

/// Merge `data-ai-meta` entries into leading YAML frontmatter. `ai_meta` overrides existing keys.
pub fn merge_ai_meta_into_markdown(md: String, ai_meta: &[(String, String)]) -> String {
    if ai_meta.is_empty() {
        return md;
    }
    let mut merged_vals: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in ai_meta {
        merged_vals.insert(k.clone(), quote_yaml_scalar(v));
    }

    if let Some((_prefix, inner, after)) = parse_simple_frontmatter_body(&md) {
        let mut base = parse_frontmatter_map(inner);
        for (k, v) in merged_vals {
            base.insert(k, v);
        }
        let fm = format_frontmatter(&base);
        return format!("{fm}{after}");
    }

    let fm_body: String = ai_meta
        .iter()
        .map(|(k, v)| format!("{}: {}\n", k, quote_yaml_scalar(v)))
        .collect();
    format!("---\n{fm_body}---\n\n{md}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_collected_via_lol_html() {
        let html = r#"<h1 data-ai-meta="title">Hello <em>world</em></h1><p data-ai-meta="version">v1</p>"#;
        let meta = collect_data_ai_meta(html).unwrap();
        assert_eq!(meta.len(), 2);
        let m: BTreeMap<_, _> = meta.into_iter().collect();
        assert_eq!(m.get("title").map(String::as_str), Some("Hello world"));
        assert_eq!(m.get("version").map(String::as_str), Some("v1"));
    }

    #[test]
    fn apply_pipeline_meta_then_strip() {
        let html = r#"<main><h1 data-ai-meta="title">T</h1></main>"#;
        let (out, meta) = apply_data_ai_html_transforms(html).unwrap();
        assert_eq!(meta.len(), 1);
        assert!(!out.contains("data-ai-meta"));
        assert!(out.contains("T"));
    }

    #[test]
    fn merge_meta_plain_scalars_match_head_meta_style() {
        let md = "---\nmeta-description: Minimal app using the arm-next package\n---\n\n# Hi\n".to_string();
        let ai = vec![
            ("author".into(), "Test 1 author".into()),
            ("canonical".into(), "Test 1 canonical".into()),
        ];
        let out = merge_ai_meta_into_markdown(md, &ai);
        assert!(out.contains("meta-description: Minimal app using the arm-next package"));
        assert!(out.contains("author: Test 1 author\n"));
        assert!(out.contains("canonical: Test 1 canonical\n"));
        assert!(!out.contains("\"Test 1 author\""));
    }

    #[test]
    fn merge_meta_quotes_ambiguous_scalars() {
        let md = "# x\n".to_string();
        assert!(
            merge_ai_meta_into_markdown(md.clone(), &[("k".into(), "a: b".into())])
                .contains("\"a: b\"")
        );
        assert!(
            merge_ai_meta_into_markdown(md, &[("k".into(), "yes".into())]).contains("\"yes\"")
        );
    }
}
