use crate::common;
use crate::error::ConverterError;
use crate::types::*;

/// 将 OpenAI Chat Completions 格式的响应转换为 Anthropic Messages 格式
pub fn convert_response(
    response: ConversionResponse,
    model_mapping: &ModelMapping,
) -> Result<ConversionResponse, ConverterError> {
    let model = model_mapping.reverse_resolve(&response.model);

    let content = response
        .content
        .iter()
        .map(common::convert_content_part_to_anthropic)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ConversionResponse {
        protocol: Protocol::AnthropicMessages,
        id: response.id,
        model,
        content,
        stop_reason: response.stop_reason.map(|r| r.to_anthropic()),
        usage: response.usage,
        metadata: response.metadata,
    })
}
