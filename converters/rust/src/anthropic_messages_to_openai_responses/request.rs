use crate::common;
use crate::error::ConverterError;
use crate::types::*;

/// 将 Anthropic Messages 格式的请求转换为 OpenAI Responses 格式
pub fn convert_request(
    request: ConversionRequest,
    model_mapping: &ModelMapping,
) -> Result<ConversionRequest, ConverterError> {
    let model = model_mapping.resolve(&request.model);

    // 将system消息合并到messages中
    let mut messages = Vec::new();

    if let Some(system) = &request.system {
        let system_text = common::extract_system_text(system);
        messages.push(Message {
            role: Role::System,
            content: Content::Text(system_text),
        });
    }

    // 转换消息
    for msg in &request.messages {
        messages.push(Message {
            role: msg.role.clone(),
            content: convert_content(&msg.content)?,
        });
    }

    let tools = request
        .tools
        .map(|tools| convert_tools(&tools))
        .transpose()?;

    Ok(ConversionRequest {
        protocol: Protocol::OpenAiResponses,
        model,
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
        tools,
        system: None, // system已合并到messages
        metadata: request.metadata,
    })
}

fn convert_content(content: &Content) -> Result<Content, ConverterError> {
    match content {
        Content::Text(text) => Ok(Content::Text(text.clone())),
        Content::Parts(parts) => {
            let converted: Vec<ContentPart> = parts
                .iter()
                .map(common::convert_content_part_to_openai)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Content::Parts(converted))
        }
    }
}

fn convert_tools(tools: &[Tool]) -> Result<Vec<Tool>, ConverterError> {
    Ok(tools.iter().map(common::normalize_tool_for_openai).collect())
}
