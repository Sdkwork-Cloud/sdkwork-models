//! ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS 转换器
//!
//! 将 Anthropic Messages API 格式转换为 OpenAI Chat Completions API 格式。
//! 典型场景：让使用 Anthropic 协议的客户端（如 Claude Code）连接 OpenAI 兼容的服务（如 DeepSeek）。

mod converter;
mod request;
mod response;

pub use converter::AnthropicMessagesToOpenAiCompletionsConverter;

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
        let converter = AnthropicMessagesToOpenAiCompletionsConverter::default();
        assert_eq!(converter.name(), "ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS");
        assert_eq!(converter.source_protocol(), Protocol::AnthropicMessages);
        assert_eq!(converter.target_protocol(), Protocol::OpenAiCompletions);
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = AnthropicMessagesToOpenAiCompletionsConverter::default();
        let req = make_request("claude-sonnet-4", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::OpenAiCompletions);
        assert_eq!(result.model, "claude-sonnet-4");
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("claude-sonnet-4".to_string(), "deepseek-v4-pro".to_string());
        mapping.insert("claude-haiku-4".to_string(), "deepseek-v4-flash".to_string());
        let converter = AnthropicMessagesToOpenAiCompletionsConverter::new(
            ModelMapping::new(mapping),
        );

        let req = make_request("claude-sonnet-4", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.model, "deepseek-v4-pro");
    }

    #[tokio::test]
    async fn test_convert_request_system_to_messages() {
        let converter = AnthropicMessagesToOpenAiCompletionsConverter::default();
        let req = make_request("claude-sonnet-4", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        // system消息应该被转换为messages中的system角色
        assert!(result.messages.iter().any(|m| m.role == Role::System));
    }
}
