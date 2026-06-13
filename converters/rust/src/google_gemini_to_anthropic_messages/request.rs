use crate::common;
use crate::error::ConverterError;
use crate::types::*;

/// 将 Google Gemini 格式的请求转换为 Anthropic Messages 格式
pub fn convert_request(
    request: ConversionRequest,
    model_mapping: &ModelMapping,
) -> Result<ConversionRequest, ConverterError> {
    let model = model_mapping.resolve(&request.model);

    let (system, messages) = common::extract_system_from_gemini_messages(&request.messages);

    Ok(ConversionRequest {
        protocol: Protocol::AnthropicMessages,
        model,
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
        tools: None,
        system,
        metadata: request.metadata,
    })
}
