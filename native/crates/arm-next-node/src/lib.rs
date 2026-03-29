#![deny(clippy::all)]

use arm_next_core::convert_pipeline;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct ConvertInput {
    pub html: String,
    #[napi(js_name = "conversionOptionsJson")]
    pub conversion_options_json: Option<String>,
    #[napi(js_name = "maxTokens")]
    pub max_tokens: Option<u32>,
    #[napi(js_name = "continuedMessage")]
    pub continued_message: Option<String>,
}

#[napi(object)]
pub struct ConvertOutput {
    pub markdown: String,
    #[napi(js_name = "tokenCount")]
    pub token_count: u32,
    pub truncated: bool,
}

#[napi(js_name = "convertPageToMarkdown")]
pub fn convert_page_to_markdown(input: ConvertInput) -> Result<ConvertOutput> {
    let continued = input.continued_message.as_deref();
    convert_pipeline(
        &input.html,
        input.conversion_options_json.as_deref(),
        input.max_tokens,
        continued,
    )
    .map(|(markdown, token_count, truncated)| ConvertOutput {
        markdown,
        token_count,
        truncated,
    })
    .map_err(|e| Error::from_reason(e))
}
