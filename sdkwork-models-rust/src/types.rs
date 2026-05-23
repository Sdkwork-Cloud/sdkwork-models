use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogManifest {
    pub name: String,
    pub schema_version: String,
    pub catalog_version: String,
    pub generated_at: String,
    pub models_root: String,
    pub schemas_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BillingMeter {
    pub meter_code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub modality: String,
    pub usage_type: Option<String>,
    pub billing_mode: Option<String>,
    pub default_unit: Option<String>,
    pub default_unit_size: String,
    pub quantity_precision: Option<i32>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolStandard {
    pub protocol_code: String,
    pub vendor_origin: String,
    pub display_name: String,
    pub family: String,
    pub docs_url: String,
    pub maturity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelVendor {
    pub vendor_code: String,
    pub region_code: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub description: Option<String>,
    pub website_url: Option<String>,
    pub docs_url: Option<String>,
    pub country_region: Option<String>,
    pub vendor_type: String,
    pub market_scope: String,
    pub billing_currency: String,
    pub billing_jurisdiction: String,
    #[serde(default)]
    pub operating_regions: Vec<String>,
    #[serde(default)]
    pub model_families: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub supported_protocols: Vec<String>,
    pub open_source: Option<bool>,
    pub sort_order: Option<i32>,
    pub source: SourceEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FamilyFile {
    pub vendor_code: String,
    pub region_code: String,
    #[serde(default)]
    pub families: Vec<ModelFamily>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelFamily {
    pub family_code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub family_type: String,
    pub primary_modality: String,
    pub default_model: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub catalog_key: String,
    pub model_id: String,
    pub display_name: String,
    pub vendor_code: String,
    pub region_code: String,
    pub vendor_name: Option<String>,
    pub family_code: String,
    pub primary_capability: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub input_modalities: Vec<String>,
    #[serde(default)]
    pub output_modalities: Vec<String>,
    pub api_format: String,
    pub context_tokens: Option<i64>,
    pub max_input_tokens: Option<i64>,
    pub max_output_tokens: Option<i64>,
    #[serde(default)]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default)]
    pub supports_json_schema: bool,
    pub rank_score: Option<String>,
    pub lifecycle: String,
    pub release_stage: String,
    pub shelf_state: String,
    pub routing_state: String,
    pub replacement_model: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub strengths: Vec<String>,
    pub color_token: Option<String>,
    pub latency_p50_ms: Option<i32>,
    pub latency_p95_ms: Option<i32>,
    pub win_rate: Option<String>,
    pub trend_score: Option<String>,
    pub source: SourceEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    pub catalog_key: String,
    pub vendor_code: String,
    pub region_code: String,
    pub model_id: String,
    pub currency: String,
    #[serde(default)]
    pub prices: Vec<ModelPrice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelPrice {
    pub price_id: String,
    pub price_side: String,
    pub pricing_scope: Option<String>,
    pub meter_code: String,
    pub unit_size: String,
    pub unit_price: String,
    pub minimum_quantity: String,
    pub quantity_step: Option<String>,
    pub currency: Option<String>,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub source: SourceEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceEvidence {
    pub source_url: String,
    pub observed_at: String,
    pub source_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankingFile {
    pub vendor_code: String,
    pub region_code: String,
    #[serde(default)]
    pub snapshots: Vec<RankingSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogIndex {
    pub schema_version: String,
    pub catalog_version: String,
    #[serde(default)]
    pub vendors: Vec<CatalogIndexVendor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogIndexVendor {
    pub vendor_code: String,
    pub region_code: String,
    pub path: String,
    pub families_path: String,
    #[serde(default)]
    pub model_files: Vec<String>,
    #[serde(default)]
    pub pricing_files: Vec<String>,
    pub rankings_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankingSnapshot {
    pub snapshot_date: String,
    pub snapshot_period: Option<String>,
    pub rank_scope: String,
    #[serde(default)]
    pub items: Vec<RankingItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankingItem {
    pub model_id: String,
    pub rank_no: i32,
    pub previous_rank_no: Option<i32>,
    pub pricing_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VendorCatalog {
    pub vendor_code: String,
    pub region_code: String,
    pub vendor: ModelVendor,
    pub families: Vec<ModelFamily>,
    pub models: Vec<ModelInfo>,
    pub pricing: Vec<ModelPricing>,
    pub rankings: Vec<RankingSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelVendorIdentity {
    pub vendor_code: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub vendor_type: String,
    pub capabilities: Vec<String>,
    pub supported_protocols: Vec<String>,
    pub open_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VendorRegionRef {
    pub vendor_code: String,
    pub region_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalog {
    pub manifest: CatalogManifest,
    pub meters: Vec<BillingMeter>,
    pub protocols: Vec<ProtocolStandard>,
    pub vendors: Vec<VendorCatalog>,
}
