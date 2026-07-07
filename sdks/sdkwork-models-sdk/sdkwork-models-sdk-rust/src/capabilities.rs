//! Model capability predicates aligned with catalog `supports*` flags and modality arrays.
//!
//! Industry alignment:
//! - Input/output modalities mirror OpenAI model modalities and Gemini input/output modalities.
//! - `tool_call` maps to OpenAI/Anthropic/Gemini function or tool calling (`supportsTools`).
//! - `structured_output` maps to JSON Schema constrained generation (`supportsJsonSchema`).
//! - `streaming` maps to streamed response delivery (`supportsStreaming`).

use crate::types::ModelInfo;

pub const MODEL_FEATURES: &[&str] = &["streaming", "tool_call", "structured_output"];

pub const MODEL_INPUT_MODALITIES: &[&str] = &["text", "image", "audio", "music", "video"];

pub const MODEL_OUTPUT_MODALITIES: &[&str] = &["text", "image", "audio", "music", "video"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCapabilityProfile {
    pub catalog_key: String,
    pub primary_capability: String,
    pub capabilities: Vec<String>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    pub features: Vec<String>,
    pub supports_streaming: bool,
    pub supports_tool_call: bool,
    pub supports_structured_output: bool,
}

pub fn get_model_capability_profile(model: &ModelInfo) -> ModelCapabilityProfile {
    ModelCapabilityProfile {
        catalog_key: model.catalog_key.clone(),
        primary_capability: model.primary_capability.clone(),
        capabilities: model.capabilities.clone(),
        input_modalities: model.input_modalities.clone(),
        output_modalities: model.output_modalities.clone(),
        features: list_model_features(model),
        supports_streaming: model_supports_streaming(model),
        supports_tool_call: model_supports_tool_call(model),
        supports_structured_output: model_supports_structured_output(model),
    }
}

pub fn model_supports_input_modality(model: &ModelInfo, modality: &str) -> bool {
    model.input_modalities.iter().any(|item| item == modality)
}

pub fn model_supports_output_modality(model: &ModelInfo, modality: &str) -> bool {
    model.output_modalities.iter().any(|item| item == modality)
}

pub fn model_supports_text_input(model: &ModelInfo) -> bool {
    model_supports_input_modality(model, "text")
}

pub fn model_supports_image_input(model: &ModelInfo) -> bool {
    model_supports_input_modality(model, "image")
}

pub fn model_supports_audio_input(model: &ModelInfo) -> bool {
    model_supports_input_modality(model, "audio")
}

pub fn model_supports_video_input(model: &ModelInfo) -> bool {
    model_supports_input_modality(model, "video")
}

pub fn model_supports_text_output(model: &ModelInfo) -> bool {
    model_supports_output_modality(model, "text")
}

pub fn model_supports_image_output(model: &ModelInfo) -> bool {
    model_supports_output_modality(model, "image")
}

pub fn model_supports_audio_output(model: &ModelInfo) -> bool {
    model_supports_output_modality(model, "audio")
}

pub fn model_supports_video_output(model: &ModelInfo) -> bool {
    model_supports_output_modality(model, "video")
}

pub fn model_supports_vision(model: &ModelInfo) -> bool {
    model_supports_image_input(model)
}

pub fn model_supports_speech_input(model: &ModelInfo) -> bool {
    model_supports_audio_input(model)
}

pub fn model_supports_speech_output(model: &ModelInfo) -> bool {
    model_supports_audio_output(model)
}

pub fn model_supports_feature(model: &ModelInfo, feature: &str) -> bool {
    match feature {
        "streaming" => model_supports_streaming(model),
        "tool_call" => model_supports_tool_call(model),
        "structured_output" => model_supports_structured_output(model),
        _ => false,
    }
}

pub fn model_supports_streaming(model: &ModelInfo) -> bool {
    model.supports_streaming
}

pub fn model_supports_tool_call(model: &ModelInfo) -> bool {
    model.supports_tools
}

pub fn model_supports_structured_output(model: &ModelInfo) -> bool {
    model.supports_json_schema
}

pub fn list_model_features(model: &ModelInfo) -> Vec<String> {
    MODEL_FEATURES
        .iter()
        .filter(|feature| model_supports_feature(model, feature))
        .map(|feature| (*feature).to_string())
        .collect()
}
