use async_trait::async_trait;

use crate::error::ConverterError;
use crate::traits::Converter;
use crate::types::*;

use super::request::convert_request;
use super::response::convert_response;

/// OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES 转换器
///
/// 将 OpenAI Chat Completions API 格式转换为 Anthropic Messages API 格式。
/// 用于 DeepSeek 等使用 OpenAI 兼容协议的 vendor 接入 Claude Code 客户端。
pub struct OpenAiCompletionsToAnthropicMessagesConverter {
    model_mapping: ModelMapping,
}

impl OpenAiCompletionsToAnthropicMessagesConverter {
    pub fn new(model_mapping: ModelMapping) -> Self {
        Self { model_mapping }
    }
}

impl Default for OpenAiCompletionsToAnthropicMessagesConverter {
    fn default() -> Self {
        Self::new(ModelMapping::default())
    }
}

#[async_trait]
impl Converter for OpenAiCompletionsToAnthropicMessagesConverter {
    fn name(&self) -> &str {
        "OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES"
    }

    fn source_protocol(&self) -> Protocol {
        Protocol::OpenAiCompletions
    }

    fn target_protocol(&self) -> Protocol {
        Protocol::AnthropicMessages
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Stream,
            Capability::Tools,
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
