use async_trait::async_trait;

use crate::error::ConverterError;
use crate::traits::Converter;
use crate::types::*;

use super::request::convert_request;
use super::response::convert_response;

/// GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES 转换器
///
/// 将 Google Gemini API 格式转换为 Anthropic Messages API 格式。
/// 用于 Gemini 客户端接入 Anthropic 兼容的服务。
pub struct GoogleGeminiToAnthropicMessagesConverter {
    model_mapping: ModelMapping,
}

impl GoogleGeminiToAnthropicMessagesConverter {
    pub fn new(model_mapping: ModelMapping) -> Self {
        Self { model_mapping }
    }
}

impl Default for GoogleGeminiToAnthropicMessagesConverter {
    fn default() -> Self {
        Self::new(ModelMapping::default())
    }
}

#[async_trait]
impl Converter for GoogleGeminiToAnthropicMessagesConverter {
    fn name(&self) -> &str {
        "GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES"
    }

    fn source_protocol(&self) -> Protocol {
        Protocol::GoogleGemini
    }

    fn target_protocol(&self) -> Protocol {
        Protocol::AnthropicMessages
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Stream,
            Capability::Tools,
            Capability::Vision,
            Capability::Audio,
            Capability::Code,
            Capability::Reasoning,
        ]
    }

    async fn convert_request(
        &self,
        request: ConversionRequest,
    ) -> Result<ConversionRequest, ConverterError> {
        convert_request(request, &self.model_mapping)
    }

    async fn convert_response(
        &self,
        response: ConversionResponse,
    ) -> Result<ConversionResponse, ConverterError> {
        convert_response(response, &self.model_mapping)
    }
}
