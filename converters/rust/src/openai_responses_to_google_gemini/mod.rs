//! OPENAI_RESPONSES_TO_GOOGLE_GEMINI 转换器

mod converter;
mod request;
mod response;

pub use converter::OpenAiResponsesToGoogleGeminiConverter;

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
            system: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_metadata() {
        let converter = OpenAiResponsesToGoogleGeminiConverter::default();
        assert_eq!(converter.name(), "OPENAI_RESPONSES_TO_GOOGLE_GEMINI");
        assert_eq!(converter.source_protocol(), Protocol::OpenAiResponses);
        assert_eq!(converter.target_protocol(), Protocol::GoogleGemini);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
        assert!(converter.capabilities().contains(&Capability::Vision));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = OpenAiResponsesToGoogleGeminiConverter::default();
        let req = make_request("gpt-5.5", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::GoogleGemini);
        assert_eq!(result.model, "gpt-5.5");
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].role, Role::User);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gpt-5.5".to_string(), "gemini-2.5-pro".to_string());
        mapping.insert("gpt-5.4-mini".to_string(), "gemini-2.5-flash".to_string());
        let converter = OpenAiResponsesToGoogleGeminiConverter::new(
            ModelMapping::new(mapping),
        );

        let req1 = make_request("gpt-5.5", "Hello");
        let result1 = converter.convert_request(req1).await.unwrap();
        assert_eq!(result1.model, "gemini-2.5-pro");

        let req2 = make_request("gpt-5.4-mini", "Hello");
        let result2 = converter.convert_request(req2).await.unwrap();
        assert_eq!(result2.model, "gemini-2.5-flash");
    }

    #[tokio::test]
    async fn test_convert_request_with_system() {
        let converter = OpenAiResponsesToGoogleGeminiConverter::default();
        let mut req = make_request("gpt-5.5", "Hello");
        req.system = Some(SystemPrompt::Text("You are helpful.".to_string()));
        let result = converter.convert_request(req).await.unwrap();
        // system消息应被转换为user消息 + assistant确认
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.messages[0].role, Role::User);
        assert_eq!(result.messages[1].role, Role::Assistant);
        assert_eq!(result.messages[2].role, Role::User);
    }

    #[tokio::test]
    async fn test_convert_request_preserves_params() {
        let converter = OpenAiResponsesToGoogleGeminiConverter::default();
        let req = make_request("gpt-5.5", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(1024));
        assert_eq!(result.temperature, Some(0.7));
        assert!(!result.stream);
    }

    #[tokio::test]
    async fn test_convert_request_system_cleared() {
        let converter = OpenAiResponsesToGoogleGeminiConverter::default();
        let mut req = make_request("gpt-5.5", "Hello");
        req.system = Some(SystemPrompt::Text("Test".to_string()));
        let result = converter.convert_request(req).await.unwrap();
        assert!(result.system.is_none());
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = OpenAiResponsesToGoogleGeminiConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::GoogleGemini,
            id: "resp_123".to_string(),
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
        assert_eq!(result.protocol, Protocol::OpenAiResponses);
        assert_eq!(result.model, "gemini-2.5-pro");
        assert_eq!(result.id, "resp_123");
    }

    #[tokio::test]
    async fn test_convert_response_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gpt-5.5".to_string(), "gemini-2.5-pro".to_string());
        let converter = OpenAiResponsesToGoogleGeminiConverter::new(
            ModelMapping::new(mapping),
        );

        let resp = ConversionResponse {
            protocol: Protocol::GoogleGemini,
            id: "resp_123".to_string(),
            model: "gemini-2.5-pro".to_string(),
            content: vec![ContentPart::Text {
                text: "Hello!".to_string(),
            }],
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage::default(),
            metadata: HashMap::new(),
        };
        let result = converter.convert_response(resp).await.unwrap();
        assert_eq!(result.model, "gpt-5.5");
    }
}
