use async_trait::async_trait;
use crate::error::ConverterError;
use crate::traits::Converter;
use crate::types::*;

use super::request::convert_request;
use super::response::convert_response;

pub struct OpenAiCompletionsToOpenAiResponsesConverter {
    model_mapping: ModelMapping,
}

impl OpenAiCompletionsToOpenAiResponsesConverter {
    pub fn new(model_mapping: ModelMapping) -> Self { Self { model_mapping } }
}

impl Default for OpenAiCompletionsToOpenAiResponsesConverter {
    fn default() -> Self { Self::new(ModelMapping::default()) }
}

#[async_trait]
impl Converter for OpenAiCompletionsToOpenAiResponsesConverter {
    fn name(&self) -> &str { "OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES" }
    fn source_protocol(&self) -> Protocol { Protocol::OpenAiCompletions }
    fn target_protocol(&self) -> Protocol { Protocol::OpenAiResponses }
    fn capabilities(&self) -> Vec<Capability> { vec![Capability::Stream, Capability::Tools, Capability::Vision, Capability::Code] }

    async fn convert_request(&self, request: ConversionRequest) -> Result<ConversionRequest, ConverterError> {
        convert_request(request, &self.model_mapping)
    }

    async fn convert_response(&self, response: ConversionResponse) -> Result<ConversionResponse, ConverterError> {
        convert_response(response, &self.model_mapping)
    }
}
