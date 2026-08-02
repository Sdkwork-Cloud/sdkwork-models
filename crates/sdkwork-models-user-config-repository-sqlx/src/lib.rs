//! Client-local user model configuration domain and repository contract.
//!
//! The user's model access channels (official / relay / custom), locally
//! persisted API keys, and per-agent-engine (tool) configurations live in a
//! client-local SQLite database fully decoupled from the server-side
//! `ai_resource` catalog tables. The authoritative schema is
//! `database-client-local/ddl/baseline/sqlite/0001_user_model_config_baseline.sql`.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const USER_MODEL_CHANNEL_KINDS: &[&str] = &["official", "relay", "custom"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserModelChannel {
    /// Stable channel code; identical to the configuration id used by agents.
    pub code: String,
    pub name: String,
    pub kind: String,
    pub base_url: String,
    pub description: String,
    pub default_vendor_code: String,
    pub default_model_id: String,
    pub api_key_configured: bool,
    pub sort_order: Option<i64>,
    pub offerings: Vec<UserModelChannelOffering>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserModelChannelOffering {
    pub vendor_code: String,
    pub vendor_name: String,
    pub models: Vec<UserModelChannelModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserModelChannelModel {
    pub model_id: String,
    pub display_name: String,
    pub context_tokens: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub tool_call_rounds: Option<i64>,
    pub supports_multimodal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserModelApiKey {
    pub channel_code: String,
    pub api_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserModelEngineConfig {
    pub engine_id: String,
    pub channel_code: String,
    pub vendor_code: String,
    pub base_url: String,
    pub default_model_id: String,
    pub supported_model_ids: Vec<String>,
    pub supported_provider_ids: Vec<String>,
    pub input_context_tokens: Option<i64>,
    pub output_context_tokens: Option<i64>,
    pub tool_call_rounds: Option<i64>,
    pub supports_multimodal: bool,
    pub api_key_configured: bool,
    pub applied_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserModelEngineSelection {
    pub engine_id: String,
    pub channel_code: String,
    pub model_id: String,
}

#[derive(Debug)]
pub enum UserModelConfigStoreError {
    Sql(sqlx::Error),
    Message(String),
}

impl fmt::Display for UserModelConfigStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sql(error) => write!(formatter, "user model config store error: {error}"),
            Self::Message(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for UserModelConfigStoreError {}

impl From<sqlx::Error> for UserModelConfigStoreError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sql(error)
    }
}

pub type UserModelConfigStoreResult<T> = Result<T, UserModelConfigStoreError>;

/// Repository contract for the client-local user model configuration store.
pub trait UserModelConfigStore: Send + Sync {
    // Channels (with offerings and models).
    fn list_channels(
        &self,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<Vec<UserModelChannel>>> + Send;
    fn get_channel(
        &self,
        code: &str,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<Option<UserModelChannel>>> + Send;
    fn upsert_channel(
        &self,
        channel: &UserModelChannel,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<()>> + Send;
    fn delete_channel(
        &self,
        code: &str,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<()>> + Send;

    // API keys.
    fn upsert_api_key(
        &self,
        key: &UserModelApiKey,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<()>> + Send;
    fn get_api_key(
        &self,
        channel_code: &str,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<Option<String>>> + Send;

    // Per-engine (tool) configurations.
    fn list_engine_configs(
        &self,
        engine_id: Option<&str>,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<Vec<UserModelEngineConfig>>> + Send;
    fn upsert_engine_config(
        &self,
        config: &UserModelEngineConfig,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<()>> + Send;

    // Per-engine selections.
    fn list_engine_selections(
        &self,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<Vec<UserModelEngineSelection>>> + Send;
    fn get_engine_selection(
        &self,
        engine_id: &str,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<Option<UserModelEngineSelection>>> + Send;
    fn upsert_engine_selection(
        &self,
        selection: &UserModelEngineSelection,
    ) -> impl std::future::Future<Output = UserModelConfigStoreResult<()>> + Send;
}

pub mod sqlite_store;
