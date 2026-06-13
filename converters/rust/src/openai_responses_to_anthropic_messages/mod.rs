//! OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES 转换器
//!
//! 将 OpenAI Responses API 格式转换为 Anthropic Messages API 格式。
//! 典型场景：让使用 OpenAI Responses 协议的客户端（如 Codex）连接 Anthropic 兼容的服务。

mod converter;
mod request;
mod response;

pub use converter::OpenAiResponsesToAnthropicMessagesConverter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Converter;
    use crate::types::*;
    use std::collections::HashMap;

    fn make_request(model: &str, user_msg: &str) -> ConversionRequest {
        ConversionRequest {
            protocol: Protocol::OpenAiResponses,
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
        let converter = OpenAiResponsesToAnthropicMessagesConverter::default();
        assert_eq!(converter.name(), "OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES");
        assert_eq!(converter.source_protocol(), Protocol::OpenAiResponses);
        assert_eq!(converter.target_protocol(), Protocol::AnthropicMessages);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = OpenAiResponsesToAnthropicMessagesConverter::default();
        let req = make_request("gpt-5.5", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::AnthropicMessages);
        assert_eq!(result.model, "gpt-5.5");
        assert!(result.system.is_some());
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].role, Role::User);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gpt-5.5".to_string(), "claude-sonnet-4".to_string());
        let converter = OpenAiResponsesToAnthropicMessagesConverter::new(
            ModelMapping::new(mapping),
        );
        let req = make_request("gpt-5.5", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.model, "claude-sonnet-4");
    }

    #[tokio::test]
    async fn test_convert_request_preserves_system() {
        let converter = OpenAiResponsesToAnthropicMessagesConverter::default();
        let req = make_request("gpt-5.5", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        match result.system {
            Some(SystemPrompt::Text(text)) => assert_eq!(text, "You are helpful."),
            _ => panic!("Expected system prompt as text"),
        }
    }

    #[tokio::test]
    async fn test_convert_request_preserves_params() {
        let converter = OpenAiResponsesToAnthropicMessagesConverter::default();
        let req = make_request("gpt-5.5", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(1024));
        assert_eq!(result.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = OpenAiResponsesToAnthropicMessagesConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::AnthropicMessages,
            id: "msg_123".to_string(),
            model: "claude-sonnet-4".to_string(),
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
        assert_eq!(result.protocol, Protocol::OpenAiResponses);
        assert_eq!(result.id, "msg_123");
        assert_eq!(result.model, "claude-sonnet-4");
    }
}
