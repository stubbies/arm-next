use arm_next_core::convert_pipeline;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct ConvertOutput {
    markdown: String,
    token_count: u32,
    truncated: bool,
}

/// Edge-safe entry: same pipeline as NAPI (prune → markdown → cl100k token guard).
/// Returns JSON: `{ "markdown", "token_count", "truncated" }`.
#[wasm_bindgen]
pub fn convert_page_to_markdown_json(
    html: &str,
    conversion_options_json: Option<String>,
    max_tokens: Option<u32>,
    continued_message: Option<String>,
) -> Result<String, JsValue> {
    let continued = continued_message.as_deref();
    convert_pipeline(
        html,
        conversion_options_json.as_deref(),
        max_tokens,
        continued,
    )
    .map(|(markdown, token_count, truncated)| {
        serde_json::to_string(&ConvertOutput {
            markdown,
            token_count,
            truncated,
        })
        .unwrap_or_else(|_| "{}".to_string())
    })
    .map_err(|e| JsValue::from_str(&e))
}
