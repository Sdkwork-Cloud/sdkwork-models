use std::collections::BTreeSet;

use serde_json::Value;

use crate::domain::{DomainError, DomainResult, ModelVendor};

pub const DEFAULT_PROVIDER_RETRY_ATTEMPTS: usize = 2;
pub const MAX_PROVIDER_RETRY_ATTEMPTS: usize = 5;
pub const MAX_PROVIDER_RETRY_BACKOFF_MS: u64 = 2_000;
pub const DEFAULT_RETRYABLE_PROVIDER_STATUS_CODES: [u16; 5] = [429, 500, 502, 503, 504];
pub const DEFAULT_PROVIDER_CIRCUIT_BREAKER_FAILURE_THRESHOLD: usize = 1;
pub const MAX_PROVIDER_CIRCUIT_BREAKER_FAILURE_THRESHOLD: usize = 100;
pub const DEFAULT_PROVIDER_CIRCUIT_BREAKER_RECOVERY_WINDOW_SECONDS: u64 = 60;

pub fn provider_native_model_id(model_key: &str) -> String {
    let value = model_key.trim();
    if value.is_empty() {
        return String::new();
    }
    if let Some(identity) = parse_model_catalog_identity(value) {
        if unambiguous_catalog_namespace_prefix(&identity.vendor_code)
            || relay_provider_namespace_prefix(&identity.vendor_code)
        {
            return identity.model_id();
        }
    }
    value.to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalogIdentity {
    pub vendor_code: String,
    pub model_parts: Vec<String>,
}

impl ModelCatalogIdentity {
    pub fn model_id(&self) -> String {
        self.model_parts.join("/")
    }
}

pub fn parse_model_catalog_identity(value: &str) -> Option<ModelCatalogIdentity> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parts = trimmed.split('/').map(str::trim).collect::<Vec<_>>();
    if parts.len() < 2 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    if is_model_region_segment(parts[1]) {
        return None;
    }
    Some(ModelCatalogIdentity {
        vendor_code: parts[0].to_owned(),
        model_parts: parts[1..].iter().map(|part| (*part).to_owned()).collect(),
    })
}

pub fn ensure_canonical_model_catalog_key(value: &str, field_name: &str) -> DomainResult<()> {
    if parse_model_catalog_identity(value).is_some() {
        return Ok(());
    }
    let parts = value.trim().split('/').map(str::trim).collect::<Vec<_>>();
    if parts.len() >= 3
        && parts
            .get(1)
            .is_some_and(|part| is_model_region_segment(part))
    {
        return Err(DomainError::new(format!(
            "{field_name} must use vendorCode/modelId; region belongs to region_code: {value}"
        )));
    }
    Err(DomainError::new(format!(
        "{field_name} must use vendorCode/modelId: {value}"
    )))
}

pub fn model_catalog_scope_matches_key(scope: &str, key: &str) -> bool {
    let scope = normalize_model_catalog_scope_value(scope);
    let key = normalize_model_catalog_scope_value(key);
    if scope.is_empty() || key.is_empty() {
        return false;
    }
    let is_valid_catalog_key = parse_model_catalog_identity(&key).is_some();
    if !is_valid_catalog_key && key.contains('/') {
        return false;
    }
    if scope == "*" || scope == "all" {
        return true;
    }
    if scope == key {
        return true;
    }
    let Some(identity) = parse_model_catalog_identity(&key) else {
        return false;
    };
    if let Some(prefix) = scope.strip_suffix("/*") {
        return !prefix.is_empty()
            && (key == prefix
                || key
                    .strip_prefix(prefix)
                    .is_some_and(|tail| tail.starts_with('/')));
    }

    let scope_parts = scope
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let native_model = identity.model_id();
    match scope_parts.as_slice() {
        [scope_value] => {
            *scope_value == identity.vendor_code
                || *scope_value == native_model.as_str()
                || identity
                    .model_parts
                    .last()
                    .is_some_and(|model| *scope_value == model.as_str())
        }
        [scope_vendor, scope_model @ ..] => {
            (*scope_vendor == identity.vendor_code
                && scope_model
                    .iter()
                    .copied()
                    .eq(identity.model_parts.iter().map(String::as_str)))
                || scope == native_model
        }
        [] => false,
    }
}

fn normalize_model_catalog_scope_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn unambiguous_catalog_namespace_prefix(prefix: &str) -> bool {
    matches!(
        prefix.trim().to_ascii_lowercase().as_str(),
        "openai"
            | "google"
            | "google_gemini"
            | "alibaba"
            | "baidu"
            | "black_forest_labs"
            | "bytedance"
            | "deepseek"
            | "elevenlabs"
            | "kuaishou"
            | "minimax"
            | "moonshot"
            | "xai"
            | "stability_ai"
            | "suno"
            | "tencent"
            | "zhipu"
    )
}

fn relay_provider_namespace_prefix(prefix: &str) -> bool {
    matches!(
        prefix.trim().to_ascii_lowercase().as_str(),
        "openrouter" | "siliconflow" | "together" | "fireworks" | "groq"
    )
}

pub fn is_model_region_segment(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        return false;
    }
    if matches!(
        value.as_str(),
        "global"
            | "cn"
            | "china"
            | "mainland"
            | "overseas"
            | "international"
            | "intl"
            | "us"
            | "eu"
            | "ap"
            | "apac"
            | "jp"
            | "sg"
            | "hk"
            | "local"
    ) {
        return true;
    }
    if is_hyphenated_cloud_region(&value) || is_china_region_alias(&value) {
        return true;
    }
    matches!(
        value.as_str(),
        "eastus"
            | "eastus2"
            | "westus"
            | "westus2"
            | "westus3"
            | "centralus"
            | "northcentralus"
            | "southcentralus"
            | "westcentralus"
            | "canadaeast"
            | "canadacentral"
            | "brazilsouth"
            | "northeurope"
            | "westeurope"
            | "francecentral"
            | "switzerlandnorth"
            | "uksouth"
            | "ukwest"
            | "swedencentral"
            | "norwayeast"
            | "germanywestcentral"
            | "italynorth"
            | "polandcentral"
            | "israelcentral"
            | "qatarcentral"
            | "uaenorth"
            | "southafricanorth"
            | "centralindia"
            | "southindia"
            | "westindia"
            | "japaneast"
            | "japanwest"
            | "koreacentral"
            | "koreasouth"
            | "eastasia"
            | "southeastasia"
            | "australiaeast"
            | "australiasoutheast"
            | "australiacentral"
            | "newzealandnorth"
            | "malaysiawest"
            | "indonesiacentral"
    )
}

fn is_hyphenated_cloud_region(value: &str) -> bool {
    let Some(prefix) = value.split('-').next() else {
        return false;
    };
    if !matches!(
        prefix,
        "af" | "ap" | "ca" | "cn" | "eu" | "il" | "me" | "sa" | "us"
    ) {
        return false;
    }
    value
        .rsplit('-')
        .next()
        .and_then(|part| part.parse::<u16>().ok())
        .is_some()
}

fn is_china_region_alias(value: &str) -> bool {
    value.strip_prefix("cn-").is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelVendorDefinition {
    pub vendor_code: String,
    pub vendor: ModelVendor,
    pub display_name: String,
}

impl ModelVendorDefinition {
    pub fn new(vendor_code: &str, vendor: ModelVendor, display_name: &str) -> Self {
        Self {
            vendor_code: vendor_code.to_owned(),
            vendor,
            display_name: display_name.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiModel {
    pub catalog_key: String,
    pub model: String,
    pub display_name: String,
    pub vendor_code: String,
    pub capabilities: Vec<String>,
    pub description: Option<String>,
    pub modalities: Vec<String>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    pub api_format: Option<String>,
    pub capability_intro: Option<String>,
    pub limitations: Vec<String>,
    pub supported_languages: Vec<String>,
    pub use_cases: Vec<String>,
    pub training_data_cutoff: Option<String>,
    pub context_tokens: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_json_schema: bool,
    pub release_stage: Option<i32>,
    pub shelf_state: Option<i32>,
    pub routing_state: Option<i32>,
    pub replacement_model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelMappingBindingType {
    ProviderAccount,
    Channel,
    ChannelGroup,
    Vendor,
    Global,
    Site,
    SiteService,
}

impl ModelMappingBindingType {
    pub fn from_str(value: &str) -> DomainResult<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "provider_account" => Ok(Self::ProviderAccount),
            "channel" => Ok(Self::Channel),
            "channel_group" => Ok(Self::ChannelGroup),
            "vendor" => Ok(Self::Vendor),
            "global" => Ok(Self::Global),
            "site" => Ok(Self::Site),
            "site_service" => Ok(Self::SiteService),
            value => Err(DomainError::new(format!(
                "ai_model_mapping_rule_binding.binding_type contains unsupported value: {value}"
            ))),
        }
    }

    pub fn priority_rank(self) -> i32 {
        match self {
            Self::ProviderAccount => 0,
            Self::Channel => 1,
            Self::ChannelGroup => 2,
            Self::Vendor => 3,
            Self::Global => 4,
            Self::Site => 5,
            Self::SiteService => 6,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProviderAccount => "provider_account",
            Self::Channel => "channel",
            Self::ChannelGroup => "channel_group",
            Self::Vendor => "vendor",
            Self::Global => "global",
            Self::Site => "site",
            Self::SiteService => "site_service",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolveModelMappingContext {
    pub vendor_code: Option<String>,
    pub channel_id: Option<i64>,
    pub channel_code: Option<String>,
    pub channel_group_id: Option<i64>,
    pub channel_group_code: Option<String>,
    pub provider_account_id: Option<i64>,
    pub provider_account_code: Option<String>,
    pub site_id: Option<i64>,
    pub site_code: Option<String>,
    pub site_service_id: Option<i64>,
    pub site_service_code: Option<String>,
}

impl ResolveModelMappingContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vendor_code(mut self, vendor_code: impl Into<String>) -> Self {
        self.vendor_code = normalized_optional_text(&vendor_code.into());
        self
    }

    pub fn with_channel_id(mut self, channel_id: i64) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    pub fn with_channel_code(mut self, channel_code: impl Into<String>) -> Self {
        self.channel_code = normalized_optional_text(&channel_code.into());
        self
    }

    pub fn with_channel_group_id(mut self, channel_group_id: i64) -> Self {
        self.channel_group_id = Some(channel_group_id);
        self
    }

    pub fn with_channel_group_code(mut self, channel_group_code: impl Into<String>) -> Self {
        self.channel_group_code = normalized_optional_text(&channel_group_code.into());
        self
    }

    pub fn with_provider_account_id(mut self, provider_account_id: i64) -> Self {
        self.provider_account_id = Some(provider_account_id);
        self
    }

    pub fn with_provider_account_code(mut self, provider_account_code: impl Into<String>) -> Self {
        self.provider_account_code = normalized_optional_text(&provider_account_code.into());
        self
    }

    pub fn with_site(mut self, site_id: Option<i64>, site_code: Option<&str>) -> Self {
        self.site_id = site_id;
        self.site_code = site_code.and_then(normalized_optional_text);
        self
    }

    pub fn with_site_service(
        mut self,
        site_service_id: Option<i64>,
        site_service_code: Option<&str>,
    ) -> Self {
        self.site_service_id = site_service_id;
        self.site_service_code = site_service_code.and_then(normalized_optional_text);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelMappingRule {
    pub id: i64,
    pub binding_type: ModelMappingBindingType,
    pub binding_id: Option<i64>,
    pub binding_code: Option<String>,
    pub source_model: String,
    pub source_catalog_key: Option<String>,
    pub target_model: String,
    pub target_catalog_key: Option<String>,
    pub target_vendor_code: Option<String>,
    pub target_provider_model: Option<String>,
    pub target_provider_native_model: Option<String>,
    pub binding_sort_order: i32,
    pub item_sort_order: i32,
}

impl ModelMappingRule {
    pub fn new(
        id: i64,
        binding_type: ModelMappingBindingType,
        source_model: &str,
        target_model: &str,
        binding_sort_order: i32,
    ) -> Self {
        Self {
            id,
            binding_type,
            binding_id: None,
            binding_code: None,
            source_model: source_model.to_owned(),
            source_catalog_key: None,
            target_model: target_model.to_owned(),
            target_catalog_key: None,
            target_vendor_code: None,
            target_provider_model: None,
            target_provider_native_model: None,
            binding_sort_order,
            item_sort_order: 100,
        }
    }

    pub fn with_binding_id(mut self, binding_id: i64) -> Self {
        self.binding_id = Some(binding_id);
        self
    }

    pub fn with_binding_code(mut self, binding_code: &str) -> Self {
        self.binding_code = normalized_optional_text(binding_code);
        self
    }

    pub fn with_source_catalog_key(mut self, source_catalog_key: &str) -> Self {
        self.source_catalog_key = normalized_optional_text(source_catalog_key);
        self
    }

    pub fn with_target_catalog_key(mut self, target_catalog_key: &str) -> Self {
        self.target_catalog_key = normalized_optional_text(target_catalog_key);
        self
    }

    pub fn with_target_vendor_code(mut self, target_vendor_code: &str) -> Self {
        self.target_vendor_code = normalized_optional_text(target_vendor_code);
        self
    }

    pub fn with_target_provider_model(mut self, target_provider_model: &str) -> Self {
        self.target_provider_model = normalized_optional_text(target_provider_model);
        self
    }

    pub fn with_target_provider_native_model(mut self, target_provider_native_model: &str) -> Self {
        self.target_provider_native_model = normalized_optional_text(target_provider_native_model);
        self
    }

    pub fn effective_catalog_key(&self) -> &str {
        self.target_catalog_key
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.target_model.as_str())
    }

    pub fn effective_provider_model(&self) -> Option<&str> {
        self.target_provider_model
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                self.target_provider_native_model
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
            })
    }
}

fn normalized_optional_text(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

impl AiModel {
    pub fn new(
        model: &str,
        display_name: &str,
        vendor_code: &str,
        capabilities: Vec<&str>,
    ) -> Self {
        Self {
            catalog_key: format!("{vendor_code}/{model}"),
            model: model.to_owned(),
            display_name: display_name.to_owned(),
            vendor_code: vendor_code.to_owned(),
            capabilities: capabilities.into_iter().map(str::to_owned).collect(),
            description: None,
            modalities: Vec::new(),
            input_modalities: Vec::new(),
            output_modalities: Vec::new(),
            api_format: None,
            capability_intro: None,
            limitations: Vec::new(),
            supported_languages: Vec::new(),
            use_cases: Vec::new(),
            training_data_cutoff: None,
            context_tokens: None,
            max_output_tokens: None,
            supports_streaming: false,
            supports_tools: false,
            supports_json_schema: false,
            release_stage: None,
            shelf_state: None,
            routing_state: None,
            replacement_model: None,
        }
    }

    pub fn with_catalog_key(mut self, catalog_key: &str) -> Self {
        self.catalog_key = catalog_key.to_owned();
        self
    }

    pub fn with_public_metadata(mut self, metadata: AiModelPublicMetadata) -> Self {
        self.description = metadata.description;
        self.modalities = metadata.modalities;
        self.input_modalities = metadata.input_modalities;
        self.output_modalities = metadata.output_modalities;
        self.api_format = metadata.api_format;
        self.capability_intro = metadata.capability_intro;
        self.limitations = metadata.limitations;
        self.supported_languages = metadata.supported_languages;
        self.use_cases = metadata.use_cases;
        self.training_data_cutoff = metadata.training_data_cutoff;
        self.context_tokens = metadata.context_tokens;
        self.max_output_tokens = metadata.max_output_tokens;
        self.supports_streaming = metadata.supports_streaming;
        self.supports_tools = metadata.supports_tools;
        self.supports_json_schema = metadata.supports_json_schema;
        self.release_stage = metadata.release_stage;
        self.shelf_state = metadata.shelf_state;
        self.routing_state = metadata.routing_state;
        self.replacement_model = metadata.replacement_model;
        self
    }

    pub fn is_publicly_active(&self) -> bool {
        matches!(self.release_stage.unwrap_or(1), 1 | 2)
            && self.shelf_state.unwrap_or(1) == 1
            && self.routing_state.unwrap_or(1) == 1
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AiModelPublicMetadata {
    pub description: Option<String>,
    pub modalities: Vec<String>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    pub api_format: Option<String>,
    pub capability_intro: Option<String>,
    pub limitations: Vec<String>,
    pub supported_languages: Vec<String>,
    pub use_cases: Vec<String>,
    pub training_data_cutoff: Option<String>,
    pub context_tokens: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_json_schema: bool,
    pub release_stage: Option<i32>,
    pub shelf_state: Option<i32>,
    pub routing_state: Option<i32>,
    pub replacement_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRetryPolicy {
    pub max_attempts: usize,
    pub retryable_status_codes: Vec<u16>,
    pub backoff_ms: u64,
}

impl ProviderRetryPolicy {
    pub fn new(
        max_attempts: usize,
        retryable_status_codes: Vec<u16>,
        backoff_ms: u64,
    ) -> DomainResult<Self> {
        if max_attempts == 0 || max_attempts > MAX_PROVIDER_RETRY_ATTEMPTS {
            return Err(DomainError::new(format!(
                "ai_channel.retry_policy max_attempts must be between 1 and {MAX_PROVIDER_RETRY_ATTEMPTS}: {max_attempts}"
            )));
        }
        if backoff_ms > MAX_PROVIDER_RETRY_BACKOFF_MS {
            return Err(DomainError::new(format!(
                "ai_channel.retry_policy backoff_ms must be <= {MAX_PROVIDER_RETRY_BACKOFF_MS}: {backoff_ms}"
            )));
        }
        if max_attempts > 1 && retryable_status_codes.is_empty() {
            return Err(DomainError::new(
                "ai_channel.retry_policy retryable_status_codes is required when max_attempts is greater than 1",
            ));
        }

        let mut seen = BTreeSet::new();
        let mut normalized = Vec::with_capacity(retryable_status_codes.len());
        for status_code in retryable_status_codes {
            if !is_allowed_retryable_provider_status(status_code) {
                return Err(DomainError::new(format!(
                    "ai_channel.retry_policy retryable_status_codes contains unsupported status: {status_code}"
                )));
            }
            if !seen.insert(status_code) {
                return Err(DomainError::new(format!(
                    "ai_channel.retry_policy retryable_status_codes contains duplicate status: {status_code}"
                )));
            }
            normalized.push(status_code);
        }

        Ok(Self {
            max_attempts,
            retryable_status_codes: normalized,
            backoff_ms,
        })
    }

    pub fn from_json_str(value: &str) -> DomainResult<Self> {
        let value: Value = serde_json::from_str(value).map_err(|error| {
            DomainError::new(format!(
                "ai_channel.retry_policy must be valid JSON: {error}"
            ))
        })?;
        let object = value
            .as_object()
            .ok_or_else(|| DomainError::new("ai_channel.retry_policy must be a JSON object"))?;
        for key in object.keys() {
            if !matches!(
                key.as_str(),
                "max_attempts" | "retryable_status_codes" | "backoff_ms"
            ) {
                return Err(DomainError::new(format!(
                    "ai_channel.retry_policy contains unsupported field: {key}"
                )));
            }
        }

        let max_attempts = object
            .get("max_attempts")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| {
                DomainError::new("ai_channel.retry_policy max_attempts must be a positive integer")
            })?;
        let retryable_status_codes = object
            .get("retryable_status_codes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                DomainError::new(
                    "ai_channel.retry_policy retryable_status_codes must be an array",
                )
            })?
            .iter()
            .map(|value| {
                value
                    .as_u64()
                    .and_then(|value| u16::try_from(value).ok())
                    .ok_or_else(|| {
                        DomainError::new(
                            "ai_channel.retry_policy retryable_status_codes must contain integer HTTP statuses",
                        )
                    })
            })
            .collect::<DomainResult<Vec<_>>>()?;
        let backoff_ms = object
            .get("backoff_ms")
            .map(|value| {
                value.as_u64().ok_or_else(|| {
                    DomainError::new(
                        "ai_channel.retry_policy backoff_ms must be a non-negative integer",
                    )
                })
            })
            .transpose()?
            .unwrap_or(0);

        Self::new(max_attempts, retryable_status_codes, backoff_ms)
    }

    pub fn is_retryable_status(&self, status_code: u16) -> bool {
        self.retryable_status_codes.contains(&status_code)
    }
}

impl Default for ProviderRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_PROVIDER_RETRY_ATTEMPTS,
            retryable_status_codes: DEFAULT_RETRYABLE_PROVIDER_STATUS_CODES.to_vec(),
            backoff_ms: 0,
        }
    }
}

fn is_allowed_retryable_provider_status(status_code: u16) -> bool {
    matches!(status_code, 408 | 409 | 425 | 429 | 500 | 502 | 503 | 504)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCircuitBreakerPolicy {
    pub failure_threshold: usize,
}

impl ProviderCircuitBreakerPolicy {
    pub fn new(failure_threshold: usize) -> DomainResult<Self> {
        if failure_threshold == 0
            || failure_threshold > MAX_PROVIDER_CIRCUIT_BREAKER_FAILURE_THRESHOLD
        {
            return Err(DomainError::new(format!(
                "ai_channel.circuit_breaker_policy failure_threshold must be between 1 and {MAX_PROVIDER_CIRCUIT_BREAKER_FAILURE_THRESHOLD}: {failure_threshold}"
            )));
        }
        Ok(Self { failure_threshold })
    }

    pub fn from_json_str(value: &str) -> DomainResult<Self> {
        let value: Value = serde_json::from_str(value).map_err(|error| {
            DomainError::new(format!(
                "ai_channel.circuit_breaker_policy must be valid JSON: {error}"
            ))
        })?;
        let object = value.as_object().ok_or_else(|| {
            DomainError::new("ai_channel.circuit_breaker_policy must be a JSON object")
        })?;
        for key in object.keys() {
            if key != "failure_threshold" {
                return Err(DomainError::new(format!(
                    "ai_channel.circuit_breaker_policy contains unsupported field: {key}"
                )));
            }
        }

        let failure_threshold = object
            .get("failure_threshold")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| {
                DomainError::new(
                    "ai_channel.circuit_breaker_policy failure_threshold must be a positive integer",
                )
            })?;
        Self::new(failure_threshold)
    }
}

impl Default for ProviderCircuitBreakerPolicy {
    fn default() -> Self {
        Self {
            failure_threshold: DEFAULT_PROVIDER_CIRCUIT_BREAKER_FAILURE_THRESHOLD,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAuthType {
    Bearer,
    Header,
    Query,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAuthHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAuthProfile {
    pub auth_type: ProviderAuthType,
    pub name: Option<String>,
    pub default_headers: Vec<ProviderAuthHeader>,
}

impl ProviderAuthProfile {
    pub fn bearer() -> Self {
        Self {
            auth_type: ProviderAuthType::Bearer,
            name: None,
            default_headers: Vec::new(),
        }
    }

    pub fn header(name: impl Into<String>) -> Self {
        Self {
            auth_type: ProviderAuthType::Header,
            name: Some(name.into()),
            default_headers: Vec::new(),
        }
    }

    pub fn query(name: impl Into<String>) -> Self {
        Self {
            auth_type: ProviderAuthType::Query,
            name: Some(name.into()),
            default_headers: Vec::new(),
        }
    }

    pub fn from_account_config(
        provider_code: &str,
        auth_type: Option<&str>,
        auth_config_json: Option<&str>,
    ) -> DomainResult<Self> {
        let config = parse_auth_config(auth_config_json)?;
        let explicit_type = auth_config_string(&config, &["type", "authType", "auth_type"])
            .or_else(|| auth_config_nested_string(&config, "auth", &["type", "authType"]))
            .or_else(|| auth_type.map(str::to_owned));
        let name = auth_config_string(
            &config,
            &[
                "name",
                "authName",
                "auth_name",
                "headerName",
                "header_name",
                "queryName",
                "query_name",
            ],
        )
        .or_else(|| {
            auth_config_nested_string(
                &config,
                "auth",
                &[
                    "name",
                    "authName",
                    "auth_name",
                    "headerName",
                    "header_name",
                    "queryName",
                    "query_name",
                ],
            )
        });
        let default_headers = parse_auth_default_headers(&config)?;

        let mut profile = match explicit_type
            .as_deref()
            .map(normalize_auth_type_code)
            .as_deref()
        {
            Some("query") => Self::query(
                name.or_else(|| default_query_auth_name(provider_code))
                    .ok_or_else(|| {
                        DomainError::new(
                            "integration_provider_account.auth_config query auth name is required",
                        )
                    })?,
            ),
            Some("header") => Self::header(
                name.or_else(|| default_header_auth_name(provider_code))
                    .ok_or_else(|| {
                        DomainError::new(
                            "integration_provider_account.auth_config header auth name is required",
                        )
                    })?,
            ),
            Some("azure_openai") => Self::header(name.unwrap_or_else(|| "api-key".to_owned())),
            Some("gcp_vertex_oauth") => Self::bearer(),
            Some("aws_bedrock") => Self::bearer(),
            Some("claude_code") => Self::bearer(),
            Some("bearer") => Self::bearer(),
            Some("standard_api_key" | "api_key" | "1" | "") | None => {
                provider_default_auth_profile(provider_code, name)
            }
            Some(value) => {
                return Err(DomainError::new(format!(
                    "integration_provider_account.auth_type contains unsupported value: {value}"
                )));
            }
        };
        validate_auth_profile(&profile)?;
        profile.default_headers = default_headers;
        validate_auth_profile(&profile)?;
        Ok(profile)
    }
}

impl Default for ProviderAuthProfile {
    fn default() -> Self {
        Self::bearer()
    }
}

fn parse_auth_config(auth_config_json: Option<&str>) -> DomainResult<Value> {
    match auth_config_json
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => serde_json::from_str(value).map_err(|error| {
            DomainError::new(format!(
                "integration_provider_account.auth_config must be valid JSON: {error}"
            ))
        }),
        None => Ok(Value::Object(Default::default())),
    }
}

fn auth_config_string(config: &Value, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        config
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

fn auth_config_nested_string(config: &Value, object_name: &str, names: &[&str]) -> Option<String> {
    config
        .get(object_name)
        .and_then(Value::as_object)
        .and_then(|object| {
            names.iter().find_map(|name| {
                object
                    .get(*name)
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
            })
        })
}

fn parse_auth_default_headers(config: &Value) -> DomainResult<Vec<ProviderAuthHeader>> {
    let Some(default_headers) = config
        .get("defaultHeaders")
        .or_else(|| config.get("default_headers"))
    else {
        return Ok(Vec::new());
    };
    let object = default_headers.as_object().ok_or_else(|| {
        DomainError::new(
            "integration_provider_account.auth_config defaultHeaders must be an object",
        )
    })?;
    let mut headers = Vec::with_capacity(object.len());
    for (name, value) in object {
        let value = value.as_str().ok_or_else(|| {
            DomainError::new(format!(
                "integration_provider_account.auth_config defaultHeaders.{name} must be a string"
            ))
        })?;
        let name = validate_provider_auth_header_name(name.trim(), "defaultHeaders header name")?;
        let value = value.trim();
        validate_provider_auth_header_value(&name, value, "defaultHeaders")?;
        headers.push(ProviderAuthHeader {
            name,
            value: value.to_owned(),
        });
    }
    headers.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(headers)
}

fn validate_auth_profile(profile: &ProviderAuthProfile) -> DomainResult<()> {
    if matches!(
        profile.auth_type,
        ProviderAuthType::Header | ProviderAuthType::Query
    ) {
        let name = profile.name.as_deref().ok_or_else(|| {
            DomainError::new("integration_provider_account.auth_config auth name is required")
        })?;
        if profile.auth_type == ProviderAuthType::Header {
            validate_provider_auth_header_name(name, "auth header name")?;
        } else if name.trim().is_empty() {
            return Err(DomainError::new(
                "integration_provider_account.auth_config query auth name must not be blank",
            ));
        }
    }
    Ok(())
}

fn validate_provider_auth_header_name(name: &str, label: &str) -> DomainResult<String> {
    let name = name.trim().to_ascii_lowercase();
    if name.is_empty() {
        return Err(DomainError::new(format!(
            "integration_provider_account.auth_config {label} must not be blank"
        )));
    }
    if !name.bytes().all(is_valid_http_header_name_byte) {
        return Err(DomainError::new(format!(
            "integration_provider_account.auth_config {label} is invalid: {name}"
        )));
    }
    Ok(name)
}

fn validate_provider_auth_header_value(name: &str, value: &str, label: &str) -> DomainResult<()> {
    if value.is_empty() {
        return Err(DomainError::new(format!(
            "integration_provider_account.auth_config {label}.{name} must not be blank"
        )));
    }
    if value.bytes().any(|byte| matches!(byte, 0..=31 | 127)) {
        return Err(DomainError::new(format!(
            "integration_provider_account.auth_config {label}.{name} contains an invalid header value"
        )));
    }
    Ok(())
}

fn is_valid_http_header_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn normalize_auth_type_code(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "2" => "gcp_vertex_oauth".to_owned(),
        "3" => "aws_bedrock".to_owned(),
        "4" => "azure_openai".to_owned(),
        "5" => "claude_code".to_owned(),
        "bearer" | "authorization_bearer" | "oauth_bearer" => "bearer".to_owned(),
        "header" | "api-key-header" | "api_key_header" => "header".to_owned(),
        "query" | "api-key-query" | "api_key_query" => "query".to_owned(),
        "azure openai" | "azure_openai" => "azure_openai".to_owned(),
        "gcp vertex oauth" | "gcp_vertex_oauth" => "gcp_vertex_oauth".to_owned(),
        "aws bedrock" | "aws_bedrock" | "sigv4" => "aws_bedrock".to_owned(),
        "claude code" | "claude_code" => "claude_code".to_owned(),
        "standard api key" | "standard_api_key" | "api key" | "api_key" | "1" | "" => {
            "standard_api_key".to_owned()
        }
        _ => value.replace(' ', "_"),
    }
}

fn provider_default_auth_profile(
    provider_code: &str,
    configured_name: Option<String>,
) -> ProviderAuthProfile {
    let provider_code = provider_code.trim().to_ascii_lowercase();
    match provider_code.as_str() {
        "google" | "gemini" | "google_gemini" => ProviderAuthProfile::header(
            configured_name.unwrap_or_else(|| "x-goog-api-key".to_owned()),
        ),
        "anthropic" | "claude" => {
            ProviderAuthProfile::header(configured_name.unwrap_or_else(|| "x-api-key".to_owned()))
        }
        "azure" | "azure_openai" => {
            ProviderAuthProfile::header(configured_name.unwrap_or_else(|| "api-key".to_owned()))
        }
        _ => configured_name
            .map(ProviderAuthProfile::header)
            .unwrap_or_else(ProviderAuthProfile::bearer),
    }
}

fn default_header_auth_name(provider_code: &str) -> Option<String> {
    match provider_code.trim().to_ascii_lowercase().as_str() {
        "google" | "gemini" | "google_gemini" => Some("x-goog-api-key".to_owned()),
        "anthropic" | "claude" => Some("x-api-key".to_owned()),
        "azure" | "azure_openai" => Some("api-key".to_owned()),
        _ => Some("x-api-key".to_owned()),
    }
}

fn default_query_auth_name(provider_code: &str) -> Option<String> {
    match provider_code.trim().to_ascii_lowercase().as_str() {
        "google" | "gemini" | "google_gemini" => Some("key".to_owned()),
        _ => Some("api_key".to_owned()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelProviderRoute {
    pub catalog_key: String,
    pub model: String,
    pub api_code: Option<String>,
    pub region_code: String,
    pub provider_code: String,
    pub channel_id: i64,
    pub credential_id: Option<i64>,
    pub credential_rotation: String,
    pub credential_priority: i32,
    pub credential_weight: i32,
    pub provider_model: String,
    pub base_url: Option<String>,
    pub secret_ref: Option<String>,
    pub auth_profile: ProviderAuthProfile,
    pub timeout_ms: Option<u64>,
    pub retry_policy: Option<ProviderRetryPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderChannelGroupBinding {
    pub group_id: i64,
    pub priority: i32,
    pub weight: i32,
    pub api_scope: Vec<String>,
    pub capabilities: Vec<String>,
}

impl ProviderChannelGroupBinding {
    pub fn new(group_id: i64, priority: i32, weight: i32) -> Self {
        Self {
            group_id,
            priority,
            weight,
            api_scope: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn new_resource_scoped<A, C, AS, CS>(
        group_id: i64,
        priority: i32,
        weight: i32,
        api_scope: AS,
        capabilities: CS,
    ) -> Self
    where
        A: Into<String>,
        C: Into<String>,
        AS: IntoIterator<Item = A>,
        CS: IntoIterator<Item = C>,
    {
        Self {
            group_id,
            priority,
            weight,
            api_scope: api_scope.into_iter().map(Into::into).collect(),
            capabilities: capabilities.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderChannelRoute {
    pub provider_code: String,
    pub channel_id: i64,
    pub credential_id: Option<i64>,
    pub credential_rotation: String,
    pub credential_priority: i32,
    pub credential_weight: i32,
    pub channel_code: Option<String>,
    pub region_code: String,
    pub site_id: Option<i64>,
    pub site_code: Option<String>,
    pub site_service_id: Option<i64>,
    pub site_service_code: Option<String>,
    pub base_url: Option<String>,
    pub secret_ref: Option<String>,
    pub auth_profile: ProviderAuthProfile,
    pub timeout_ms: Option<u64>,
    pub retry_policy: Option<ProviderRetryPolicy>,
    pub group_bindings: Vec<ProviderChannelGroupBinding>,
    pub channel_health_status: i32,
    pub credential_health_status: i32,
}

impl ProviderChannelRoute {
    pub fn new(provider_code: &str, channel_id: i64) -> Self {
        Self {
            provider_code: provider_code.to_owned(),
            channel_id,
            credential_id: None,
            credential_rotation: DEFAULT_CREDENTIAL_ROTATION.to_owned(),
            credential_priority: 100,
            credential_weight: 100,
            channel_code: None,
            region_code: "global".to_owned(),
            site_id: None,
            site_code: None,
            site_service_id: None,
            site_service_code: None,
            base_url: None,
            secret_ref: None,
            auth_profile: ProviderAuthProfile::default(),
            timeout_ms: None,
            retry_policy: None,
            group_bindings: Vec::new(),
            channel_health_status: 1,
            credential_health_status: 1,
        }
    }

    pub fn with_channel_code(mut self, channel_code: &str) -> Self {
        self.channel_code = normalized_optional_text(channel_code);
        self
    }

    pub fn with_credential(
        mut self,
        credential_id: Option<i64>,
        credential_rotation: impl Into<String>,
        credential_priority: i32,
        credential_weight: i32,
    ) -> Self {
        self.credential_id = credential_id.filter(|value| *value > 0);
        self.credential_rotation = normalize_credential_rotation_or_default(credential_rotation);
        self.credential_priority = credential_priority;
        self.credential_weight = credential_weight.max(0);
        self
    }

    pub fn with_region_code(mut self, region_code: &str) -> Self {
        self.region_code = normalized_model_region_code(region_code);
        self
    }

    pub fn with_site(mut self, site_id: Option<i64>, site_code: Option<&str>) -> Self {
        self.site_id = site_id;
        self.site_code = site_code.and_then(normalized_optional_text);
        self
    }

    pub fn with_site_service(
        mut self,
        site_service_id: Option<i64>,
        site_service_code: Option<&str>,
    ) -> Self {
        self.site_service_id = site_service_id;
        self.site_service_code = site_service_code.and_then(normalized_optional_text);
        self
    }

    pub fn with_provider_endpoint(
        mut self,
        base_url: Option<impl Into<String>>,
        secret_ref: Option<impl Into<String>>,
    ) -> Self {
        self.base_url = base_url.map(Into::into);
        self.secret_ref = secret_ref.map(Into::into);
        self
    }

    pub fn with_auth_profile(mut self, auth_profile: ProviderAuthProfile) -> Self {
        self.auth_profile = auth_profile;
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_retry_policy(mut self, retry_policy: ProviderRetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    pub fn with_group_binding(mut self, group_id: i64, priority: i32, weight: i32) -> Self {
        self.group_bindings
            .push(ProviderChannelGroupBinding::new(group_id, priority, weight));
        self
    }

    pub fn with_resource_scoped_group_binding<A, C, AS, CS>(
        mut self,
        group_id: i64,
        priority: i32,
        weight: i32,
        api_scope: AS,
        capabilities: CS,
    ) -> Self
    where
        A: Into<String>,
        C: Into<String>,
        AS: IntoIterator<Item = A>,
        CS: IntoIterator<Item = C>,
    {
        self.group_bindings
            .push(ProviderChannelGroupBinding::new_resource_scoped(
                group_id,
                priority,
                weight,
                api_scope,
                capabilities,
            ));
        self
    }

    pub fn with_group_bindings(mut self, group_bindings: Vec<ProviderChannelGroupBinding>) -> Self {
        self.group_bindings = group_bindings;
        self
    }

    /// Returns true when both the channel and its credential are healthy (health_status == 1).
    pub fn is_channel_healthy(&self) -> bool {
        self.channel_health_status == 1 && self.credential_health_status == 1
    }
}

impl ModelProviderRoute {
    pub fn new(model: &str, provider_code: &str, channel_id: i64, provider_model: &str) -> Self {
        Self {
            catalog_key: model.to_owned(),
            model: model.to_owned(),
            api_code: None,
            region_code: "global".to_owned(),
            provider_code: provider_code.to_owned(),
            channel_id,
            credential_id: None,
            credential_rotation: DEFAULT_CREDENTIAL_ROTATION.to_owned(),
            credential_priority: 100,
            credential_weight: 100,
            provider_model: provider_model.to_owned(),
            base_url: None,
            secret_ref: None,
            auth_profile: ProviderAuthProfile::default(),
            timeout_ms: None,
            retry_policy: None,
        }
    }

    pub fn new_for_catalog_key(
        catalog_key: &str,
        model: &str,
        provider_code: &str,
        channel_id: i64,
        provider_model: &str,
    ) -> Self {
        Self {
            catalog_key: catalog_key.to_owned(),
            model: model.to_owned(),
            api_code: None,
            region_code: "global".to_owned(),
            provider_code: provider_code.to_owned(),
            channel_id,
            credential_id: None,
            credential_rotation: DEFAULT_CREDENTIAL_ROTATION.to_owned(),
            credential_priority: 100,
            credential_weight: 100,
            provider_model: provider_model.to_owned(),
            base_url: None,
            secret_ref: None,
            auth_profile: ProviderAuthProfile::default(),
            timeout_ms: None,
            retry_policy: None,
        }
    }

    pub fn with_catalog_key(mut self, catalog_key: &str) -> Self {
        self.catalog_key = catalog_key.to_owned();
        self
    }

    pub fn with_api_code(mut self, api_code: &str) -> Self {
        let api_code = api_code.trim();
        self.api_code = (!api_code.is_empty()).then(|| api_code.to_owned());
        self
    }

    pub fn with_region_code(mut self, region_code: &str) -> Self {
        self.region_code = normalized_model_region_code(region_code);
        self
    }

    pub fn with_credential(
        mut self,
        credential_id: Option<i64>,
        credential_rotation: impl Into<String>,
        credential_priority: i32,
        credential_weight: i32,
    ) -> Self {
        self.credential_id = credential_id.filter(|value| *value > 0);
        self.credential_rotation = normalize_credential_rotation_or_default(credential_rotation);
        self.credential_priority = credential_priority;
        self.credential_weight = credential_weight.max(0);
        self
    }

    pub fn with_provider_endpoint(
        mut self,
        base_url: Option<impl Into<String>>,
        secret_ref: Option<impl Into<String>>,
    ) -> Self {
        self.base_url = base_url.map(Into::into);
        self.secret_ref = secret_ref.map(Into::into);
        self
    }

    pub fn with_auth_profile(mut self, auth_profile: ProviderAuthProfile) -> Self {
        self.auth_profile = auth_profile;
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_retry_policy(mut self, retry_policy: ProviderRetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }
}

fn normalized_model_region_code(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "global".to_owned()
    } else {
        value.to_owned()
    }
}

const DEFAULT_CREDENTIAL_ROTATION: &str = "default";

fn normalize_credential_rotation_or_default(value: impl Into<String>) -> String {
    match value.into().trim().to_ascii_lowercase().as_str() {
        "priority" => "priority".to_owned(),
        "round_robin" => "round_robin".to_owned(),
        "weighted_round_robin" => "weighted_round_robin".to_owned(),
        "random" => "random".to_owned(),
        _ => DEFAULT_CREDENTIAL_ROTATION.to_owned(),
    }
}
