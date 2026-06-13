//! ANTHROPIC_MESSAGES_TO_OPENAI_RESPONSES 转换器
//!
//! 将 Anthropic Messages API 格式转换为 OpenAI Responses API 格式。
//! 典型场景：让使用 Anthropic 协议的客户端（如 Claude Code）连接 OpenAI Responses 兼容的服务。

mod converter;
mod request;
mod response;

pub use converter::AnthropicMessagesToOpenAiResponsesConverter;

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
        let converter = AnthropicMessagesToOpenAiResponsesConverter::default();
        assert_eq!(converter.name(), "ANTHROPIC_MESSAGES_TO_OPENAI_RESPONSES");
        assert_eq!(converter.source_protocol(), Protocol::AnthropicMessages);
        assert_eq!(converter.target_protocol(), Protocol::OpenAiResponses);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = AnthropicMessagesToOpenAiResponsesConverter::default();
        let req = make_request("claude-sonnet-4", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::OpenAiResponses);
        assert_eq!(result.model, "claude-sonnet-4");
        // system消息被合并到messages中，所以长度为2
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].role, Role::System);
        assert_eq!(result.messages[1].role, Role::User);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("claude-sonnet-4".to_string(), "gpt-5.5".to_string());
        mapping.insert("claude-haiku-4".to_string(), "gpt-5.4-mini".to_string());
        let converter = AnthropicMessagesToOpenAiResponsesConverter::new(
            ModelMapping::new(mapping),
        );

        let req1 = make_request("claude-sonnet-4", "Hello");
        let result1 = converter.convert_request(req1).await.unwrap();
        assert_eq!(result1.model, "gpt-5.5");

        let req2 = make_request("claude-haiku-4", "Hello");
        let result2 = converter.convert_request(req2).await.unwrap();
        assert_eq!(result2.model, "gpt-5.4-mini");
    }

    #[tokio::test]
    async fn test_convert_request_with_system() {
        let converter = AnthropicMessagesToOpenAiResponsesConverter::default();
        let req = make_request("claude-sonnet-4", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        // system should be extracted to messages
        assert!(result.system.is_none() || result.messages.iter().any(|m| m.role == Role::System));
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = AnthropicMessagesToOpenAiResponsesConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::OpenAiResponses,
            id: "msg_123".to_string(),
            model: "gpt-5.5".to_string(),
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
        assert_eq!(result.model, "gpt-5.5");
    }
}
