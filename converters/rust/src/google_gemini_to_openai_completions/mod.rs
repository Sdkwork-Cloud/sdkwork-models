//! GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS 转换器

mod converter;
mod request;
mod response;

pub use converter::GoogleGeminiToOpenAiCompletionsConverter;

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
        let converter = GoogleGeminiToOpenAiCompletionsConverter::default();
        assert_eq!(converter.name(), "GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS");
        assert_eq!(converter.source_protocol(), Protocol::GoogleGemini);
        assert_eq!(converter.target_protocol(), Protocol::OpenAiCompletions);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = GoogleGeminiToOpenAiCompletionsConverter::default();
        let req = make_request("gemini-2.5-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::OpenAiCompletions);
        assert_eq!(result.model, "gemini-2.5-pro");
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gemini-2.5-pro".to_string(), "deepseek-v4-pro".to_string());
        let converter = GoogleGeminiToOpenAiCompletionsConverter::new(
            ModelMapping::new(mapping),
        );

        let req = make_request("gemini-2.5-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.model, "deepseek-v4-pro");
    }

    #[tokio::test]
    async fn test_convert_request_with_system_marker() {
        let converter = GoogleGeminiToOpenAiCompletionsConverter::default();
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
    async fn test_convert_request_default_max_tokens() {
        let converter = GoogleGeminiToOpenAiCompletionsConverter::default();
        let mut req = make_request("gemini-2.5-pro", "Hello");
        req.max_tokens = None;
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(4096));
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = GoogleGeminiToOpenAiCompletionsConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::OpenAiCompletions,
            id: "resp_123".to_string(),
            model: "deepseek-v4-pro".to_string(),
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
        assert_eq!(result.model, "deepseek-v4-pro");
    }

    #[tokio::test]
    async fn test_convert_response_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("gemini-2.5-pro".to_string(), "deepseek-v4-pro".to_string());
        let converter = GoogleGeminiToOpenAiCompletionsConverter::new(
            ModelMapping::new(mapping),
        );

        let resp = ConversionResponse {
            protocol: Protocol::OpenAiCompletions,
            id: "resp_123".to_string(),
            model: "deepseek-v4-pro".to_string(),
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
