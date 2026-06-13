//! OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES 转换器
//!
//! 将 OpenAI Chat Completions API 格式转换为 OpenAI Responses API 格式。
//! 典型场景：让使用 Completions API 的客户端连接 Responses API 兼容的服务。

mod converter;
mod request;
mod response;

pub use converter::OpenAiCompletionsToOpenAiResponsesConverter;

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
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::default();
        assert_eq!(converter.name(), "OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES");
        assert_eq!(converter.source_protocol(), Protocol::OpenAiCompletions);
        assert_eq!(converter.target_protocol(), Protocol::OpenAiResponses);
        assert!(converter.capabilities().contains(&Capability::Stream));
        assert!(converter.capabilities().contains(&Capability::Tools));
    }

    #[tokio::test]
    async fn test_convert_request_basic() {
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::default();
        let req = make_request("deepseek-v4-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.protocol, Protocol::OpenAiResponses);
        assert_eq!(result.model, "deepseek-v4-pro");
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("deepseek-v4-pro".to_string(), "gpt-5.5".to_string());
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::new(
            ModelMapping::new(mapping),
        );
        let req = make_request("deepseek-v4-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.model, "gpt-5.5");
    }

    #[tokio::test]
    async fn test_convert_request_extract_system() {
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::default();
        let mut req = make_request("deepseek-v4-pro", "Hello");
        req.messages.insert(0, Message {
            role: Role::System,
            content: Content::Text("You are helpful.".to_string()),
        });
        let result = converter.convert_request(req).await.unwrap();
        // system消息应被提取到system字段
        assert!(result.system.is_some());
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_convert_request_preserves_params() {
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::default();
        let req = make_request("deepseek-v4-pro", "Hello");
        let result = converter.convert_request(req).await.unwrap();
        assert_eq!(result.max_tokens, Some(1024));
        assert_eq!(result.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_convert_request_preserves_tools() {
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::default();
        let mut req = make_request("deepseek-v4-pro", "Hello");
        req.tools = Some(vec![Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "test".to_string(),
                description: Some("Test function".to_string()),
                parameters: Some(serde_json::json!({"type": "object"})),
                input_schema: None,
            },
        }]);
        let result = converter.convert_request(req).await.unwrap();
        assert!(result.tools.is_some());
    }

    #[tokio::test]
    async fn test_convert_response_basic() {
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::default();
        let resp = ConversionResponse {
            protocol: Protocol::OpenAiResponses,
            id: "resp_123".to_string(),
            model: "gpt-5.5".to_string(),
            content: vec![ContentPart::Text {
                text: "Hello!".to_string(),
            }],
            stop_reason: Some(StopReason::Stop),
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
        assert_eq!(result.model, "gpt-5.5");
    }

    #[tokio::test]
    async fn test_convert_response_with_model_mapping() {
        let mut mapping = HashMap::new();
        mapping.insert("deepseek-v4-pro".to_string(), "gpt-5.5".to_string());
        let converter = OpenAiCompletionsToOpenAiResponsesConverter::new(
            ModelMapping::new(mapping),
        );
        let resp = ConversionResponse {
            protocol: Protocol::OpenAiResponses,
            id: "resp_123".to_string(),
            model: "gpt-5.5".to_string(),
            content: vec![ContentPart::Text {
                text: "Hello!".to_string(),
            }],
            stop_reason: Some(StopReason::Stop),
            usage: Usage::default(),
            metadata: HashMap::new(),
        };
        let result = converter.convert_response(resp).await.unwrap();
        assert_eq!(result.model, "deepseek-v4-pro");
    }
}
