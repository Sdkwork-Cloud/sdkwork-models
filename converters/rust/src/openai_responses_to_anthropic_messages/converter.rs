use async_trait::async_trait;

use crate::error::ConverterError;
use crate::traits::Converter;
use crate::types::*;

use super::request::convert_request;
use super::response::convert_response;

/// OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES 转换器
///
/// 将 OpenAI Responses API 格式转换为 Anthropic Messages API 格式。
/// 支持流式输出、工具调用、视觉能力。
pub struct OpenAiResponsesToAnthropicMessagesConverter {
    model_mapping: ModelMapping,
}

impl OpenAiResponsesToAnthropicMessagesConverter {
    pub fn new(model_mapping: ModelMapping) -> Self {
        Self { model_mapping }
    }
}

impl Default for OpenAiResponsesToAnthropicMessagesConverter {
    fn default() -> Self {
        Self::new(ModelMapping::default())
    }
}

#[async_trait]
impl Converter for OpenAiResponsesToAnthropicMessagesConverter {
    fn name(&self) -> &str {
        "OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES"
    }

    fn source_protocol(&self) -> Protocol {
        Protocol::OpenAiResponses
    }

    fn target_protocol(&self) -> Protocol {
        Protocol::AnthropicMessages
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Stream,
            Capability::Tools,
            Capability::Vision,
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
