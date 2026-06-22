use crate::common;
use crate::error::ConverterError;
use crate::types::*;

/// 将 OpenAI Responses 格式的请求转换为 Google Gemini 格式
pub fn convert_request(
    request: ConversionRequest,
    model_mapping: &ModelMapping,
) -> Result<ConversionRequest, ConverterError> {
    let model = model_mapping.resolve(&request.model);

    // 将system消息合并到messages中
    let mut messages = Vec::new();

    if let Some(system) = &request.system {
        let system_text = common::extract_system_text(system);
        messages.extend(common::encode_system_for_gemini(&system_text));
    }

    // 转换消息
    for msg in &request.messages {
        let role = match msg.role {
            Role::Assistant => Role::Assistant,
            _ => Role::User,
        };
        messages.push(Message {
            role,
            content: convert_content(&msg.content)?,
        });
    }

    common::reject_unconverted_tools(&request.tools)?;

    Ok(ConversionRequest {
        protocol: Protocol::GoogleGemini,
        model,
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
        tools: None,
        system: None,
        metadata: request.metadata,
    })
}

fn convert_content(content: &Content) -> Result<Content, ConverterError> {
    match content {
        Content::Text(text) => Ok(Content::Text(text.clone())),
        Content::Parts(parts) => {
            let converted: Vec<ContentPart> = parts
                .iter()
                .map(common::convert_content_part_passthrough)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Content::Parts(converted))
        }
    }
}
