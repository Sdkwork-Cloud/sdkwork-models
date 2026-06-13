//! OPENAI_COMPLETIONS_TO_GOOGLE_GEMINI 转换器

mod converter;
mod request;
mod response;

pub use converter::OpenAiCompletionsToGoogleGeminiConverter;

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
        let converter = OpenAiCompletionsToGoogleGeminiConverter::default();
        assert_eq!(converter.name(), "OPENAI_COMPLETIONS_TO_GOOGLE_GEMINI");
        assert_eq!(converter.source_protocol(), Protocol::OpenAiCompletions);
        assert_eq!(converter.target_protocol(), Protocol::GoogleGemini);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = OpenAiCompletionsToGoogleGeminiConverter::default();
        let req = make_request("deepseek-v4-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::GoogleGemini);
        assert_eq!(result.model, "deepseek-v4-pro");
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("deepseek-v4-pro".to_string(), "gemini-2.5-pro".to_string());
        let converter = OpenAiCompletionsToGoogleGeminiConverter::new(
            ModelMapping::new(mapping),
        );

        let req = make_request("deepseek-v4-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.model, "gemini-2.5-pro");
    }

    #[tokio::test]
    async fn test_convert_request_with_system() {
        let converter = OpenAiCompletionsToGoogleGeminiConverter::default();
        let mut req = make_request("deepseek-v4-pro", "Hello");
        req.system = Some(SystemPrompt::Text("You are helpful.".to_string()));
        let result = converter.convert_request(req).await.unwrap();
        // system应被转换为user消息 + assistant确认
        assert_eq!(result.messages.len(), 3);
        assert!(result.system.is_none());
    }

    #[tokio::test]
    async fn test_convert_request_preserves_params() {
        let converter = OpenAiCompletionsToGoogleGeminiConverter::default();
        let req = make_request("deepseek-v4-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(1024));
        assert_eq!(result.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = OpenAiCompletionsToGoogleGeminiConverter::default();
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
        assert_eq!(result.protocol, Protocol::OpenAiCompletions);
        assert_eq!(result.model, "gemini-2.5-pro");
    }

    #[tokio::test]
    async fn test_convert_response_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("deepseek-v4-pro".to_string(), "gemini-2.5-pro".to_string());
        let converter = OpenAiCompletionsToGoogleGeminiConverter::new(
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
        assert_eq!(result.model, "deepseek-v4-pro");
    }
}
