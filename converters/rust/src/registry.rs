use std::collections::HashMap;
use std::sync::Arc;

use crate::error::ConverterError;
use crate::traits::Converter;
use crate::types::Protocol;

/// 转换器注册表
pub struct ConverterRegistry {
    converters: HashMap<String, Arc<dyn Converter>>,
}

impl ConverterRegistry {
    pub fn new() -> Self {
        Self {
            converters: HashMap::new(),
        }
    }

    /// 注册转换器
    pub fn register(&mut self, converter: Arc<dyn Converter>) {
        let name = converter.name().to_string();
        self.converters.insert(name, converter);
    }

    /// 注销转换器
    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn Converter>> {
        self.converters.remove(name)
    }

    /// 获取转换器
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Converter>> {
        self.converters.get(name)
    }

    /// 按协议对查找转换器
    pub fn find(&self, source: &Protocol, target: &Protocol) -> Option<&Arc<dyn Converter>> {
        self.converters
            .values()
            .find(|c| c.can_convert(source, target))
    }

    /// 列出所有已注册的转换器名称
    pub fn list(&self) -> Vec<&str> {
        self.converters.keys().map(|k| k.as_str()).collect()
    }

    /// 按源协议筛选
    pub fn by_source_protocol(&self, protocol: &Protocol) -> Vec<&Arc<dyn Converter>> {
        self.converters
            .values()
            .filter(|c| c.source_protocol() == *protocol)
            .collect()
    }

    /// 按目标协议筛选
    pub fn by_target_protocol(&self, protocol: &Protocol) -> Vec<&Arc<dyn Converter>> {
        self.converters
            .values()
            .filter(|c| c.target_protocol() == *protocol)
            .collect()
    }

    /// 执行转换请求
    pub async fn convert_request(
        &self,
        converter_name: &str,
        request: crate::types::ConversionRequest,
    ) -> Result<crate::types::ConversionRequest, ConverterError> {
        let converter = self
            .get(converter_name)
            .ok_or_else(|| ConverterError::InternalError(format!("Converter not found: {}", converter_name)))?;
        converter.convert_request(request).await
    }

    /// 执行转换响应
    pub async fn convert_response(
        &self,
        converter_name: &str,
        response: crate::types::ConversionResponse,
    ) -> Result<crate::types::ConversionResponse, ConverterError> {
        let converter = self
            .get(converter_name)
            .ok_or_else(|| ConverterError::InternalError(format!("Converter not found: {}", converter_name)))?;
        converter.convert_response(response).await
    }

    /// 注册所有内置转换器
    pub fn register_defaults(&mut self) {
        use crate::anthropic_messages_to_google_gemini::AnthropicMessagesToGoogleGeminiConverter;
        use crate::anthropic_messages_to_openai_completions::AnthropicMessagesToOpenAiCompletionsConverter;
        use crate::anthropic_messages_to_openai_responses::AnthropicMessagesToOpenAiResponsesConverter;
        use crate::google_gemini_to_anthropic_messages::GoogleGeminiToAnthropicMessagesConverter;
        use crate::google_gemini_to_openai_completions::GoogleGeminiToOpenAiCompletionsConverter;
        use crate::google_gemini_to_openai_responses::GoogleGeminiToOpenAiResponsesConverter;
        use crate::openai_completions_to_anthropic_messages::OpenAiCompletionsToAnthropicMessagesConverter;
        use crate::openai_completions_to_google_gemini::OpenAiCompletionsToGoogleGeminiConverter;
        use crate::openai_completions_to_openai_responses::OpenAiCompletionsToOpenAiResponsesConverter;
        use crate::openai_responses_to_anthropic_messages::OpenAiResponsesToAnthropicMessagesConverter;
        use crate::openai_responses_to_google_gemini::OpenAiResponsesToGoogleGeminiConverter;
        use crate::openai_responses_to_openai_completions::OpenAiResponsesToOpenAiCompletionsConverter;

        // OpenAI Responses →
        self.register(Arc::new(OpenAiResponsesToAnthropicMessagesConverter::default()));
        self.register(Arc::new(OpenAiResponsesToOpenAiCompletionsConverter::default()));
        self.register(Arc::new(OpenAiResponsesToGoogleGeminiConverter::default()));

        // OpenAI Completions →
        self.register(Arc::new(OpenAiCompletionsToAnthropicMessagesConverter::default()));
        self.register(Arc::new(OpenAiCompletionsToOpenAiResponsesConverter::default()));
        self.register(Arc::new(OpenAiCompletionsToGoogleGeminiConverter::default()));

        // Anthropic Messages →
        self.register(Arc::new(AnthropicMessagesToOpenAiResponsesConverter::default()));
        self.register(Arc::new(AnthropicMessagesToOpenAiCompletionsConverter::default()));
        self.register(Arc::new(AnthropicMessagesToGoogleGeminiConverter::default()));

        // Google Gemini →
        self.register(Arc::new(GoogleGeminiToAnthropicMessagesConverter::default()));
        self.register(Arc::new(GoogleGeminiToOpenAiResponsesConverter::default()));
        self.register(Arc::new(GoogleGeminiToOpenAiCompletionsConverter::default()));
    }
}

impl Default for ConverterRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }
}
