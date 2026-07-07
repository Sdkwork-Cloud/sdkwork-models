use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

pub mod bundled;
pub mod capabilities;
pub mod loader;
pub mod query;
pub mod types;
pub mod validation;

pub use capabilities::{
    get_model_capability_profile, list_model_features, model_supports_audio_input,
    model_supports_audio_output, model_supports_feature, model_supports_image_input,
    model_supports_image_output, model_supports_input_modality, model_supports_output_modality,
    model_supports_speech_input, model_supports_speech_output, model_supports_streaming,
    model_supports_structured_output, model_supports_text_input, model_supports_text_output,
    model_supports_tool_call, model_supports_video_input, model_supports_video_output,
    model_supports_vision, ModelCapabilityProfile, MODEL_FEATURES, MODEL_INPUT_MODALITIES,
    MODEL_OUTPUT_MODALITIES,
};

pub use loader::{load_bundled_catalog, load_catalog, load_vendor_catalog};
pub use query::{
    catalog_key, find_meter, find_model, find_model_by_vendor_region, find_protocol,
    find_video_profile, get_best_reference_price, get_model_prices, get_model_region_prices,
    list_available_models, list_client_api_compatibility_by_vendor, list_meters, list_models,
    list_models_by_capability, list_models_by_modality, list_models_by_protocol,
    list_models_for_voice, list_models_with_feature, list_protocols, list_protocols_by_vendor,
    list_vendor_regions, list_vendors, list_video_profiles, list_video_profiles_for_model,
    list_voices, list_voices_for_model, video_profile_catalog_key, ModelFilter, VideoProfileFilter,
    VoiceFilter,
};
pub use types::*;
pub use validation::{validate_catalog, CatalogIssue};

#[derive(Debug)]
pub enum CatalogError {
    Io(io::Error),
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl Display for CatalogError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "catalog IO error: {error}"),
            Self::Json { path, source } => {
                write!(
                    formatter,
                    "catalog JSON error in {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for CatalogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json { source, .. } => Some(source),
        }
    }
}

impl From<io::Error> for CatalogError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
