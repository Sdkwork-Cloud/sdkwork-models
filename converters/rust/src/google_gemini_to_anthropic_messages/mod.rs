//! GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES 转换器
//!
//! 将 Google Gemini API 格式转换为 Anthropic Messages API 格式。
//! 典型场景：让使用 Google Gemini 协议的客户端连接 Anthropic 兼容的服务。

mod converter;
mod request;
mod response;

pub use converter::GoogleGeminiToAnthropicMessagesConverter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Converter;
    use crate::types::*;
    use std::collections::HashMap;

    fn make_request(model: &str, user_msg: &str) -> ConversionRequest {
        ConversionRequest {
            protocol: Protocol::GoogleGemini,
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
        let converter = GoogleGeminiToAnthropicMessagesConverter::default();
        assert_eq!(converter.name(), "GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES");
        assert_eq!(converter.source_protocol(), Protocol::GoogleGemini);
        assert_eq!(converter.target_protocol(), Protocol::AnthropicMessages);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
        assert!(converter.capabilities().contains(&Capability::Vision));
        assert!(converter.capabilities().contains(&Capability::Audio));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = GoogleGeminiToAnthropicMessagesConverter::default();
        let req = make_request("gemini-2.5-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::AnthropicMessages);
        assert_eq!(result.model, "gemini-2.5-pro");
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gemini-2.5-pro".to_string(), "claude-sonnet-4".to_string());
        mapping.insert("gemini-2.5-flash".to_string(), "claude-haiku-4".to_string());
        let converter = GoogleGeminiToAnthropicMessagesConverter::new(
            ModelMapping::new(mapping),
        );

        let req1 = make_request("gemini-2.5-pro", "Hello");
        let result1 = converter.convert_request(req1).await.unwrap();
        assert_eq!(result1.model, "claude-sonnet-4");

        let req2 = make_request("gemini-2.5-flash", "Hello");
        let result2 = converter.convert_request(req2).await.unwrap();
        assert_eq!(result2.model, "claude-haiku-4");
    }

    #[tokio::test]
    async fn test_convert_request_system_extraction() {
        let converter = GoogleGeminiToAnthropicMessagesConverter::default();
        let mut req = make_request("gemini-2.5-pro", "Hello");
        req.messages.insert(0, Message {
            role: Role::User,
            content: Content::Text("[System: You are helpful.]".to_string()),
        });
        req.messages.insert(1, Message {
            role: Role::Assistant,
            content: Content::Text("Understood. I will follow these instructions.".to_string()),
        });
        let result = converter.convert_request(req).await.unwrap();
        assert!(result.system.is_some());
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_preserves_params() {
        let converter = GoogleGeminiToAnthropicMessagesConverter::default();
        let req = make_request("gemini-2.5-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(1024));
        assert_eq!(result.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_convert_request_multimodal_content() {
        let converter = GoogleGeminiToAnthropicMessagesConverter::default();
        let mut req = make_request("gemini-2.5-pro", "What is in this image?");
        req.messages[0].content = Content::Parts(vec![
            ContentPart::Text {
                text: "What is in this image?".to_string(),
            },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: "https://example.com/image.jpg".to_string(),
                    detail: Some("auto".to_string()),
                },
            },
        ]);
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.messages.len(), 1);
        match &result.messages[0].content {
            Content::Parts(parts) => assert_eq!(parts.len(), 2),
            _ => panic!("Expected Parts content"),
        }
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = GoogleGeminiToAnthropicMessagesConverter::default();
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
        assert_eq!(result.protocol, Protocol::GoogleGemini);
        assert_eq!(result.model, "claude-sonnet-4");
    }

    #[tokio::test]
    async fn test_convert_response_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gemini-2.5-pro".to_string(), "claude-sonnet-4".to_string());
        let converter = GoogleGeminiToAnthropicMessagesConverter::new(
            ModelMapping::new(mapping),
        );

        let resp = ConversionResponse {
            protocol: Protocol::AnthropicMessages,
            id: "msg_123".to_string(),
            model: "claude-sonnet-4".to_string(),
            content: vec![ContentPart::Text {
                text: "Hello!".to_string(),
            }],
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage::default(),
            metadata: HashMap::new(),
        };
        let result = converter.convert_response(resp).await.unwrap();
        assert_eq!(result.model, "gemini-2.5-pro");
    }
}
