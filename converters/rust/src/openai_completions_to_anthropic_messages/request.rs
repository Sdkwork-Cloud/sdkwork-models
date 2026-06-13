use crate::common;
use crate::error::ConverterError;
use crate::types::*;

/// 将 OpenAI Chat Completions 格式的请求转换为 Anthropic Messages 格式
pub fn convert_request(
    request: ConversionRequest,
    model_mapping: &ModelMapping,
) -> Result<ConversionRequest, ConverterError> {
    let model = model_mapping.resolve(&request.model);

    let system = request.system;

    let messages = convert_messages(&request.messages)?;

    let tools = request
        .tools
        .map(|tools| convert_tools(&tools))
        .transpose()?;

    Ok(ConversionRequest {
        protocol: Protocol::AnthropicMessages,
        model,
        messages,
        max_tokens: request.max_tokens.or(Some(4096)),
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
        tools,
        system,
        metadata: request.metadata,
    })
}

fn convert_messages(messages: &[Message]) -> Result<Vec<Message>, ConverterError> {
    let mut converted = Vec::new();
    let mut consecutive_assistant = false;

    for msg in messages {
        if msg.role == Role::System {
            continue;
        }

        if msg.role == Role::Assistant && consecutive_assistant {
            continue;
        }

        consecutive_assistant = msg.role == Role::Assistant;

        match &msg.role {
            Role::Tool => {
                let content = convert_tool_content(&msg.content)?;
                converted.push(Message {
                    role: Role::User,
                    content: Content::Parts(vec![ContentPart::ToolResult {
                        tool_use_id: extract_tool_call_id(&msg.content),
                        content: ToolResultContent::Text(content),
                    }]),
                });
            }
            _ => {
                let content = convert_content(&msg.content)?;
                converted.push(Message {
                    role: msg.role.clone(),
                    content,
                });
            }
        }
    }

    if converted.is_empty() {
        return Err(ConverterError::invalid_request(
            "At least one non-system message is required",
        ));
    }

    Ok(converted)
}

fn convert_content(content: &Content) -> Result<Content, ConverterError> {
    match content {
        Content::Text(text) => Ok(Content::Text(text.clone())),
        Content::Parts(parts) => {
            let converted: Vec<ContentPart> = parts
                .iter()
                .map(common::convert_content_part_to_anthropic)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Content::Parts(converted))
        }
    }
}

fn convert_tools(tools: &[Tool]) -> Result<Vec<Tool>, ConverterError> {
    Ok(tools.iter().map(common::normalize_tool_for_anthropic).collect())
}

fn convert_tool_content(content: &Content) -> Result<String, ConverterError> {
    match content {
        Content::Text(text) => Ok(text.clone()),
        Content::Parts(parts) => {
            let text: String = parts
                .iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            Ok(text)
        }
    }
}

fn extract_tool_call_id(content: &Content) -> String {
    match content {
        Content::Parts(parts) => parts
            .iter()
            .find_map(|p| match p {
                ContentPart::ToolResult { tool_use_id, .. } => Some(tool_use_id.clone()),
                _ => None,
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}
