use crate::common;
use crate::error::ConverterError;
use crate::types::*;

/// 将 OpenAI Responses 格式的请求转换为 OpenAI Completions 格式
pub fn convert_request(
    request: ConversionRequest,
    model_mapping: &ModelMapping,
) -> Result<ConversionRequest, ConverterError> {
    let model = model_mapping.resolve(&request.model);

    // Responses API 可以有 system 字段，Completions API 需要作为消息传递
    let mut messages = Vec::new();

    if let Some(system) = &request.system {
        let system_text = common::extract_system_text(system);
        messages.push(Message {
            role: Role::System,
            content: Content::Text(system_text),
        });
    }

    messages.extend(request.messages);

    Ok(ConversionRequest {
        protocol: Protocol::OpenAiCompletions,
        model,
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
        tools: request.tools,
        system: None,
        metadata: request.metadata,
    })
}
