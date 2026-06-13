use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConverterError {
    #[error("Unsupported conversion: {from} -> {to}")]
    UnsupportedConversion { from: String, to: String },

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Model mapping not found: {0}")]
    ModelMappingNotFound(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ConverterError {
    pub fn unsupported_conversion(from: &str, to: &str) -> Self {
        Self::UnsupportedConversion {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    pub fn invalid_response(msg: impl Into<String>) -> Self {
        Self::InvalidResponse(msg.into())
    }

    pub fn missing_field(field: &str) -> Self {
        Self::MissingField(field.to_string())
    }
}
