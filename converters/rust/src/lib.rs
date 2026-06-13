//! # SDKWork Client API Protocol Converters
//!
//! 核心转换器框架，提供不同客户端API协议之间的转换能力。
//!
//! ## 支持的转换
//!
//! | 转换器名称 | 源协议 | 目标协议 | 说明 |
//! |------------|--------|----------|------|
//! | OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES | OpenAI Responses | Anthropic Messages | Codex → Claude Code |
//! | OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS | OpenAI Responses | OpenAI Completions | Codex → DeepSeek |
//! | OPENAI_RESPONSES_TO_GOOGLE_GEMINI | OpenAI Responses | Google Gemini | Codex → Gemini |
//! | OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES | OpenAI Completions | Anthropic Messages | DeepSeek → Claude Code |
//! | OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES | OpenAI Completions | OpenAI Responses | DeepSeek → Codex |
//! | OPENAI_COMPLETIONS_TO_GOOGLE_GEMINI | OpenAI Completions | Google Gemini | DeepSeek → Gemini |
//! | ANTHROPIC_MESSAGES_TO_OPENAI_RESPONSES | Anthropic Messages | OpenAI Responses | Claude Code → Codex |
//! | ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS | Anthropic Messages | OpenAI Completions | Claude Code → DeepSeek |
//! | ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI | Anthropic Messages | Google Gemini | Claude Code → Gemini |
//! | GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES | Google Gemini | Anthropic Messages | Gemini → Claude Code |
//! | GOOGLE_GEMINI_TO_OPENAI_RESPONSES | Google Gemini | OpenAI Responses | Gemini → Codex |
//! | GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS | Google Gemini | OpenAI Completions | Gemini → DeepSeek |
//!
//! ## 使用示例
//!
//! ```rust
//! use std::sync::Arc;
//! use sdkwork_converters::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ConverterError> {
//!     let mut registry = ConverterRegistry::new();
//!     registry.register_defaults();
//!
//!     let converter = registry.get("OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES").unwrap();
//!     let request = ConversionRequest::default();
//!     let _response = converter.convert_request(request).await;
//!     Ok(())
//! }
//! ```

pub mod common;
pub mod error;
pub mod mappers;
pub mod registry;
pub mod traits;
pub mod types;

// OpenAI Responses → Anthropic Messages
pub mod openai_responses_to_anthropic_messages;

// OpenAI Responses → OpenAI Completions
pub mod openai_responses_to_openai_completions;

// OpenAI Responses → Google Gemini
pub mod openai_responses_to_google_gemini;

// OpenAI Completions → Anthropic Messages
pub mod openai_completions_to_anthropic_messages;

// OpenAI Completions → OpenAI Responses
pub mod openai_completions_to_openai_responses;

// OpenAI Completions → Google Gemini
pub mod openai_completions_to_google_gemini;

// Anthropic Messages → OpenAI Responses
pub mod anthropic_messages_to_openai_responses;

// Anthropic Messages → OpenAI Completions
pub mod anthropic_messages_to_openai_completions;

// Anthropic Messages → Google Gemini
pub mod anthropic_messages_to_google_gemini;

// Google Gemini → Anthropic Messages
pub mod google_gemini_to_anthropic_messages;

// Google Gemini → OpenAI Responses
pub mod google_gemini_to_openai_responses;

// Google Gemini → OpenAI Completions
pub mod google_gemini_to_openai_completions;

pub mod prelude {
    pub use crate::error::ConverterError;
    pub use crate::mappers::{Mapper, ModelMapper};
    pub use crate::registry::ConverterRegistry;
    pub use crate::traits::Converter;
    pub use crate::types::*;

    // 所有转换器
    pub use crate::anthropic_messages_to_google_gemini::AnthropicMessagesToGoogleGeminiConverter;
    pub use crate::anthropic_messages_to_openai_completions::AnthropicMessagesToOpenAiCompletionsConverter;
    pub use crate::anthropic_messages_to_openai_responses::AnthropicMessagesToOpenAiResponsesConverter;
    pub use crate::google_gemini_to_anthropic_messages::GoogleGeminiToAnthropicMessagesConverter;
    pub use crate::google_gemini_to_openai_completions::GoogleGeminiToOpenAiCompletionsConverter;
    pub use crate::google_gemini_to_openai_responses::GoogleGeminiToOpenAiResponsesConverter;
    pub use crate::openai_completions_to_anthropic_messages::OpenAiCompletionsToAnthropicMessagesConverter;
    pub use crate::openai_completions_to_google_gemini::OpenAiCompletionsToGoogleGeminiConverter;
    pub use crate::openai_completions_to_openai_responses::OpenAiCompletionsToOpenAiResponsesConverter;
    pub use crate::openai_responses_to_anthropic_messages::OpenAiResponsesToAnthropicMessagesConverter;
    pub use crate::openai_responses_to_google_gemini::OpenAiResponsesToGoogleGeminiConverter;
    pub use crate::openai_responses_to_openai_completions::OpenAiResponsesToOpenAiCompletionsConverter;
}
