//! ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI 转换器
//!
//! 将 Anthropic Messages API 格式转换为 Google Gemini API 格式。
//! 典型场景：让使用 Anthropic 协议的客户端（如 Claude Code）连接 Google Gemini 服务。

mod converter;
mod request;
mod response;

pub use converter::AnthropicMessagesToGoogleGeminiConverter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Converter;
    use crate::types::*;
    use std::collections::HashMap;

    fn make_request(model: &str, user_msg: &str) -> ConversionRequest {
        ConversionRequest {
            protocol: Protocol::AnthropicMessages,
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
            system: Some(SystemPrompt::Text("You are helpful.".to_string())),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_metadata() {
        let converter = AnthropicMessagesToGoogleGeminiConverter::default();
        assert_eq!(converter.name(), "ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI");
        assert_eq!(converter.source_protocol(), Protocol::AnthropicMessages);
        assert_eq!(converter.target_protocol(), Protocol::GoogleGemini);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
        assert!(converter.capabilities().contains(&Capability::Vision));
        assert!(converter.capabilities().contains(&Capability::Audio));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = AnthropicMessagesToGoogleGeminiConverter::default();
        let mut req = make_request("claude-sonnet-4", "Hello");
        req.system = None; // 不包含system消息
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::GoogleGemini);
        assert_eq!(result.model, "claude-sonnet-4");
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("claude-sonnet-4".to_string(), "gemini-2.5-pro".to_string());
        mapping.insert("claude-haiku-4".to_string(), "gemini-2.5-flash".to_string());
        let converter = AnthropicMessagesToGoogleGeminiConverter::new(
            ModelMapping::new(mapping),
        );

        let req1 = make_request("claude-sonnet-4", "Hello");
        let result1 = converter.convert_request(req1).await.unwrap();
        assert_eq!(result1.model, "gemini-2.5-pro");

        let req2 = make_request("claude-haiku-4", "Hello");
        let result2 = converter.convert_request(req2).await.unwrap();
        assert_eq!(result2.model, "gemini-2.5-flash");
    }

    #[tokio::test]
    async fn test_convert_request_system_to_messages() {
        let converter = AnthropicMessagesToGoogleGeminiConverter::default();
        let req = make_request("claude-sonnet-4", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        // system应被转换为user消息 + assistant确认
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.messages[0].role, Role::User);
        assert_eq!(result.messages[1].role, Role::Assistant);
        assert_eq!(result.messages[2].role, Role::User);
        assert!(result.system.is_none());
    }

    #[tokio::test]
    async fn test_convert_request_preserves_params() {
        let converter = AnthropicMessagesToGoogleGeminiConverter::default();
        let req = make_request("claude-sonnet-4", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(1024));
        assert_eq!(result.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_convert_request_with_tools() {
        let converter = AnthropicMessagesToGoogleGeminiConverter::default();
        let mut req = make_request("claude-sonnet-4", "What is the weather?");
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
        // Gemini暂时不转换工具
        assert!(result.tools.is_none());
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = AnthropicMessagesToGoogleGeminiConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::GoogleGemini,
            id: "msg_123".to_string(),
            model: "gemini-2.5-pro".to_string(),
            content: vec![ContentPart::Text {
                text: "Hello!".to_string(),
            }],
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
            metadata: HashMap::new(),
        };
        let result = converter.convert_response(resp).await.unwrap();
        assert_eq!(result.protocol, Protocol::AnthropicMessages);
        assert_eq!(result.model, "gemini-2.5-pro");
        assert_eq!(result.id, "msg_123");
    }

    #[tokio::test]
    async fn test_convert_response_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("claude-sonnet-4".to_string(), "gemini-2.5-pro".to_string());
        let converter = AnthropicMessagesToGoogleGeminiConverter::new(
            ModelMapping::new(mapping),
        );

        let resp = ConversionResponse {
            protocol: Protocol::GoogleGemini,
            id: "msg_123".to_string(),
            model: "gemini-2.5-pro".to_string(),
            content: vec![ContentPart::Text {
                text: "Hello!".to_string(),
            }],
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage::default(),
            metadata: HashMap::new(),
        };
        let result = converter.convert_response(resp).await.unwrap();
        assert_eq!(result.model, "claude-sonnet-4");
    }

    #[tokio::test]
    async fn test_convert_response_preserves_content() {
        let converter = AnthropicMessagesToGoogleGeminiConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::GoogleGemini,
            id: "msg_123".to_string(),
            model: "gemini-2.5-pro".to_string(),
            content: vec![
                ContentPart::Text {
                    text: "Here is the result:".to_string(),
                },
                ContentPart::Text {
                    text: "42".to_string(),
                },
            ],
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage::default(),
            metadata: HashMap::new(),
        };
        let result = converter.convert_response(resp).await.unwrap();
        assert_eq!(result.content.len(), 2);
    }
}
