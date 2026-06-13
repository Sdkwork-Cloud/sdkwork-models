//! OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES 转换器
//!
//! 将 OpenAI Chat Completions API 格式转换为 Anthropic Messages API 格式。
//! 典型场景：让使用 OpenAI 兼容协议的客户端（如 DeepSeek）连接 Anthropic 兼容的服务。

mod converter;
mod request;
mod response;

pub use converter::OpenAiCompletionsToAnthropicMessagesConverter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Converter;
    use crate::types::*;
    use std::collections::HashMap;

    fn make_request(model: &str, user_msg: &str) -> ConversionRequest {
        ConversionRequest {
            protocol: Protocol::OpenAiCompletions,
            model: model.to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Content::Text(user_msg.to_string()),
            }],
            max_tokens: Some(1024),
            temperature: Some(0.7),
            top_p: None,
            stream: false,
            tools: None,
            system: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_metadata() {
        let converter = OpenAiCompletionsToAnthropicMessagesConverter::default();
        assert_eq!(
            converter.name(),
            "OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES"
        );
        assert_eq!(converter.source_protocol(), Protocol::OpenAiCompletions);
        assert_eq!(converter.target_protocol(), Protocol::AnthropicMessages);
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = OpenAiCompletionsToAnthropicMessagesConverter::default();
        let req = make_request("deepseek-v4-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::AnthropicMessages);
        assert_eq!(result.model, "deepseek-v4-pro");
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("deepseek-v4-pro".to_string(), "claude-sonnet-4".to_string());
        mapping.insert("deepseek-v4-flash".to_string(), "claude-haiku-4".to_string());
        let converter = OpenAiCompletionsToAnthropicMessagesConverter::new(
            ModelMapping::new(mapping),
        );

        let req1 = make_request("deepseek-v4-pro", "Hello");
        let result1 = converter.convert_request(req1).await.unwrap();
        assert_eq!(result1.model, "claude-sonnet-4");

        let req2 = make_request("deepseek-v4-flash", "Hello");
        let result2 = converter.convert_request(req2).await.unwrap();
        assert_eq!(result2.model, "claude-haiku-4");
    }

    #[tokio::test]
    async fn test_convert_request_system_message() {
        let converter = OpenAiCompletionsToAnthropicMessagesConverter::default();
        let mut req = make_request("deepseek-v4-pro", "Hello");
        req.system = Some(SystemPrompt::Text("You are helpful.".to_string()));
        let result = converter.convert_request(req).await.unwrap();
        assert!(result.system.is_some());
    }

    #[tokio::test]
    async fn test_convert_request_with_tools() {
        let converter = OpenAiCompletionsToAnthropicMessagesConverter::default();
        let mut req = make_request("deepseek-v4-pro", "What is the weather?");
        req.tools = Some(vec![Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "get_weather".to_string(),
                description: Some("Get weather information".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": { "type": "string" }
                    }
                })),
                input_schema: None,
            },
        }]);
        let result = converter.convert_request(req).await.unwrap();
        assert!(result.tools.is_some());
        assert_eq!(result.tools.as_ref().unwrap().len(), 1);
    }
}
