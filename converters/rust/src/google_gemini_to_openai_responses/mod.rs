//! GOOGLE_GEMINI_TO_OPENAI_RESPONSES 转换器

mod converter;
mod request;
mod response;

pub use converter::GoogleGeminiToOpenAiResponsesConverter;

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
        let converter = GoogleGeminiToOpenAiResponsesConverter::default();
        assert_eq!(converter.name(), "GOOGLE_GEMINI_TO_OPENAI_RESPONSES");
        assert_eq!(converter.source_protocol(), Protocol::GoogleGemini);
        assert_eq!(converter.target_protocol(), Protocol::OpenAiResponses);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = GoogleGeminiToOpenAiResponsesConverter::default();
        let req = make_request("gemini-2.5-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::OpenAiResponses);
        assert_eq!(result.model, "gemini-2.5-pro");
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gemini-2.5-pro".to_string(), "gpt-5.5".to_string());
        let converter = GoogleGeminiToOpenAiResponsesConverter::new(
            ModelMapping::new(mapping),
        );

        let req = make_request("gemini-2.5-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.model, "gpt-5.5");
    }

    #[tokio::test]
    async fn test_convert_request_with_system_marker() {
        let converter = GoogleGeminiToOpenAiResponsesConverter::default();
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
        // 系统消息应被提取到system字段，确认消息应被跳过
        assert!(result.system.is_some());
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_preserves_params() {
        let converter = GoogleGeminiToOpenAiResponsesConverter::default();
        let req = make_request("gemini-2.5-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(1024));
        assert_eq!(result.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = GoogleGeminiToOpenAiResponsesConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::OpenAiResponses,
            id: "resp_123".to_string(),
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
        assert_eq!(result.protocol, Protocol::GoogleGemini);
        assert_eq!(result.model, "gpt-5.5");
    }

    #[tokio::test]
    async fn test_convert_response_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gemini-2.5-pro".to_string(), "gpt-5.5".to_string());
        let converter = GoogleGeminiToOpenAiResponsesConverter::new(
            ModelMapping::new(mapping),
        );

        let resp = ConversionResponse {
            protocol: Protocol::OpenAiResponses,
            id: "resp_123".to_string(),
            model: "gpt-5.5".to_string(),
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
