use crate::error::ConverterError;
use crate::types::*;

/// 将 OpenAI Completions 格式的请求转换为 OpenAI Responses 格式
pub fn convert_request(
    request: ConversionRequest,
    model_mapping: &ModelMapping,
) -> Result<ConversionRequest, ConverterError> {
    let model = model_mapping.resolve(&request.model);

    // 从messages中提取system消息
    let mut system = None;
    let mut messages = Vec::new();

    for msg in &request.messages {
        if msg.role == Role::System && system.is_none() {
            system = Some(SystemPrompt::Text(match &msg.content {
                Content::Text(text) => text.clone(),
                Content::Parts(parts) => parts
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(""),
            }));
        } else {
            messages.push(msg.clone());
        }
    }

    Ok(ConversionRequest {
        protocol: Protocol::OpenAiResponses,
        model,
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
        tools: request.tools,
        system,
        metadata: request.metadata,
    })
}
