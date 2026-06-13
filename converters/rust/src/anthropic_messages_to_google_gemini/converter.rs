use async_trait::async_trait;

use crate::error::ConverterError;
use crate::traits::Converter;
use crate::types::*;

use super::request::convert_request;
use super::response::convert_response;

/// ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI 转换器
///
/// 将 Anthropic Messages API 格式转换为 Google Gemini API 格式。
/// 用于 Claude Code 等使用 Anthropic 协议的客户端接入 Google Gemini 服务。
pub struct AnthropicMessagesToGoogleGeminiConverter {
    model_mapping: ModelMapping,
}

impl AnthropicMessagesToGoogleGeminiConverter {
    pub fn new(model_mapping: ModelMapping) -> Self {
        Self { model_mapping }
    }
}

impl Default for AnthropicMessagesToGoogleGeminiConverter {
    fn default() -> Self {
        Self::new(ModelMapping::default())
    }
}

#[async_trait]
impl Converter for AnthropicMessagesToGoogleGeminiConverter {
    fn name(&self) -> &str {
        "ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI"
    }

    fn source_protocol(&self) -> Protocol {
        Protocol::AnthropicMessages
    }

    fn target_protocol(&self) -> Protocol {
        Protocol::GoogleGemini
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
