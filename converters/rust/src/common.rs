//! 公共工具函数
//!
//! 提取自多个转换器模块的通用功能。

use crate::error::ConverterError;
use crate::types::*;

/// 从 SystemPrompt 中提取文本内容
pub fn extract_system_text(system: &SystemPrompt) -> String {
    match system {
        SystemPrompt::Text(text) => text.clone(),
        SystemPrompt::Parts(parts) => parts
            .iter()
            .filter_map(|p| match p {
                SystemContentPart::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(""),
    }
}

/// 解析 data URL 为 (media_type, base64_data)
///
/// 格式: `data:<media_type>;base64,<data>`
pub fn parse_data_url(url: &str) -> Result<(String, String), ConverterError> {
    let rest = url
        .strip_prefix("data:")
        .ok_or_else(|| ConverterError::invalid_request("Invalid data URL format"))?;

    let parts: Vec<&str> = rest.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(ConverterError::invalid_request("Invalid data URL format"));
    }

    let meta = parts[0];
    let data = parts[1].to_string();

    let media_type = meta
        .strip_suffix(";base64")
        .unwrap_or(meta)
        .to_string();

    Ok((media_type, data))
}

/// 将 base64 ImageSource 转换为 data URL ImageUrl
pub fn image_source_to_data_url(source: &ImageSource) -> ImageUrl {
    ImageUrl {
        url: format!("data:{};base64,{}", source.media_type, source.data),
        detail: Some("auto".to_string()),
    }
}

/// 将 data URL ImageUrl 转换为 base64 ImageSource
pub fn data_url_to_image_source(image_url: &ImageUrl) -> Result<ImageSource, ConverterError> {
    let (media_type, data) = parse_data_url(&image_url.url)?;
    Ok(ImageSource {
        source_type: "base64".to_string(),
        media_type,
        data,
    })
}

/// 将 Thinking 内容转换为带标签的文本
pub fn thinking_to_tagged_text(thinking: &str) -> String {
    format!("<thinking>{}</thinking>", thinking)
}

/// 将工具定义规范化为 OpenAI 格式（确保 parameters 字段存在）
pub fn normalize_tool_for_openai(tool: &Tool) -> Tool {
    Tool {
        tool_type: tool.tool_type.clone(),
        function: Function {
            name: tool.function.name.clone(),
            description: tool.function.description.clone(),
            parameters: tool
                .function
                .parameters
                .clone()
                .or_else(|| tool.function.input_schema.clone()),
            input_schema: tool.function.input_schema.clone(),
        },
    }
}

/// 将工具定义规范化为 Anthropic 格式（确保 input_schema 字段存在）
pub fn normalize_tool_for_anthropic(tool: &Tool) -> Tool {
    Tool {
        tool_type: tool.tool_type.clone(),
        function: Function {
            name: tool.function.name.clone(),
            description: tool.function.description.clone(),
            parameters: tool.function.parameters.clone(),
            input_schema: tool
                .function
                .input_schema
                .clone()
                .or_else(|| tool.function.parameters.clone()),
        },
    }
}

/// 将 Anthropic 系统消息编码为 Gemini 兼容的消息序列
pub fn encode_system_for_gemini(system_text: &str) -> Vec<Message> {
    vec![
        Message {
            role: Role::User,
            content: Content::Text(format!("[System: {}]", system_text)),
        },
        Message {
            role: Role::Assistant,
            content: Content::Text("Understood. I will follow these instructions.".to_string()),
        },
    ]
}

/// 从 Gemini 消息中检测并提取系统消息标记
pub fn extract_system_from_gemini_messages(
    messages: &[Message],
) -> (Option<SystemPrompt>, Vec<Message>) {
    let mut system = None;
    let mut filtered = Vec::new();
    let mut skip_next_confirmation = false;

    for msg in messages {
        if skip_next_confirmation {
            skip_next_confirmation = false;
            continue;
        }

        if msg.role == Role::User {
            if let Content::Text(text) = &msg.content {
                if text.starts_with("[System: ") && text.ends_with(']') {
                    let system_text = &text[9..text.len() - 1];
                    system = Some(SystemPrompt::Text(system_text.to_string()));
                    skip_next_confirmation = true;
                    continue;
                }
            }
        }

        if msg.role == Role::Assistant {
            if let Content::Text(text) = &msg.content {
                if text == "Understood. I will follow these instructions." {
                    continue;
                }
            }
        }

        filtered.push(msg.clone());
    }

    (system, filtered)
}

/// 转换内容部分（通用）
pub fn convert_content_part_passthrough(part: &ContentPart) -> Result<ContentPart, ConverterError> {
    match part {
        ContentPart::Text { text } => Ok(ContentPart::Text { text: text.clone() }),
        ContentPart::ImageUrl { image_url } => Ok(ContentPart::ImageUrl {
            image_url: image_url.clone(),
        }),
        ContentPart::Image { source } => Ok(ContentPart::Image {
            source: source.clone(),
        }),
        ContentPart::ToolUse { id, name, input } => Ok(ContentPart::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
        }),
        ContentPart::ToolResult {
            tool_use_id,
            content,
        } => Ok(ContentPart::ToolResult {
            tool_use_id: tool_use_id.clone(),
            content: content.clone(),
        }),
        ContentPart::Thinking { thinking } => Ok(ContentPart::Thinking {
            thinking: thinking.clone(),
        }),
    }
}

/// 转换内容部分为 Anthropic 格式（Image data URL -> base64）
pub fn convert_content_part_to_anthropic(part: &ContentPart) -> Result<ContentPart, ConverterError> {
    match part {
        ContentPart::ImageUrl { image_url } => {
            if image_url.url.starts_with("data:") {
                let source = data_url_to_image_source(image_url)?;
                Ok(ContentPart::Image { source })
            } else {
                Ok(ContentPart::ImageUrl {
                    image_url: image_url.clone(),
                })
            }
        }
        ContentPart::Thinking { thinking } => Ok(ContentPart::Thinking {
            thinking: thinking.clone(),
        }),
        _ => convert_content_part_passthrough(part),
    }
}

/// 转换内容部分为 OpenAI 格式（Image base64 -> data URL）
pub fn convert_content_part_to_openai(part: &ContentPart) -> Result<ContentPart, ConverterError> {
    match part {
        ContentPart::Image { source } => {
            let image_url = image_source_to_data_url(source);
            Ok(ContentPart::ImageUrl { image_url })
        }
        ContentPart::Thinking { thinking } => Ok(ContentPart::Text {
            text: thinking_to_tagged_text(thinking),
        }),
        _ => convert_content_part_passthrough(part),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_url() {
        let url = "data:image/png;base64,iVBORw0KGgo=";
        let (media_type, data) = parse_data_url(url).unwrap();
        assert_eq!(media_type, "image/png");
        assert_eq!(data, "iVBORw0KGgo=");
    }

    #[test]
    fn test_parse_data_url_without_base64() {
        let url = "data:text/plain,hello";
        let (media_type, data) = parse_data_url(url).unwrap();
        assert_eq!(media_type, "text/plain");
        assert_eq!(data, "hello");
    }

    #[test]
    fn test_extract_system_text() {
        let system = SystemPrompt::Text("You are helpful.".to_string());
        assert_eq!(extract_system_text(&system), "You are helpful.");
    }

    #[test]
    fn test_thinking_to_tagged_text() {
        let result = thinking_to_tagged_text("Let me think...");
        assert_eq!(result, "<thinking>Let me think...</thinking>");
    }

    #[test]
    fn test_normalize_tool_for_openai() {
        let tool = Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "test".to_string(),
                description: None,
                parameters: None,
                input_schema: Some(serde_json::json!({"type": "object"})),
            },
        };
        let normalized = normalize_tool_for_openai(&tool);
        assert!(normalized.function.parameters.is_some());
    }

    #[test]
    fn test_normalize_tool_for_anthropic() {
        let tool = Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "test".to_string(),
                description: None,
                parameters: Some(serde_json::json!({"type": "object"})),
                input_schema: None,
            },
        };
        let normalized = normalize_tool_for_anthropic(&tool);
        assert!(normalized.function.input_schema.is_some());
    }

    #[test]
    fn test_encode_system_for_gemini() {
        let messages = encode_system_for_gemini("Test system");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, Role::User);
        assert_eq!(messages[1].role, Role::Assistant);
    }

    #[test]
    fn test_extract_system_from_gemini_messages() {
        let messages = vec![
            Message {
                role: Role::User,
                content: Content::Text("[System: You are helpful.]".to_string()),
            },
            Message {
                role: Role::Assistant,
                content: Content::Text("Understood. I will follow these instructions.".to_string()),
            },
            Message {
                role: Role::User,
                content: Content::Text("Hello".to_string()),
            },
        ];
        let (system, filtered) = extract_system_from_gemini_messages(&messages);
        assert!(system.is_some());
        assert_eq!(filtered.len(), 1);
    }
}
