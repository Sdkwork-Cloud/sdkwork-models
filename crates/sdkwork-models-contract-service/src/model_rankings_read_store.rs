use std::future::Future;
use std::pin::Pin;

use serde::Serialize;

use crate::{DomainError, DomainResult};

pub const DEFAULT_MODEL_RANKING_RANK_SCOPE: &str = "commercial-default";
pub const DEFAULT_MODEL_RANKING_SNAPSHOT_PERIOD: &str = "daily";

pub type ModelRankingsReadFuture<'a> =
    Pin<Box<dyn Future<Output = DomainResult<ModelRankingsSnapshot>> + Send + 'a>>;

pub type ModelRankingRefreshStatusReadFuture<'a> =
    Pin<Box<dyn Future<Output = DomainResult<ModelRankingRefreshStatus>> + Send + 'a>>;

pub type ModelRankingRefreshJobHistoryReadFuture<'a> =
    Pin<Box<dyn Future<Output = DomainResult<ModelRankingRefreshJobHistoryPage>> + Send + 'a>>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelRankingsQuery {
    pub rank_scope: Option<String>,
    pub vendor_code: Option<String>,
    pub modality: Option<String>,
    pub search_query: Option<String>,
    pub limit: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelRankingRefreshStatusQuery {
    pub rank_scope: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelRankingRefreshJobHistoryQuery {
    pub rank_scope: Option<String>,
    pub limit: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelRankingsSubject {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub user_id: i64,
}

pub fn normalize_scope_ids(tenant_id: i64, organization_id: i64) -> (i64, i64) {
    let tenant_id = tenant_id.max(0);
    let organization_id = if tenant_id <= 0 {
        0
    } else {
        organization_id.max(0)
    };
    (tenant_id, organization_id)
}

pub fn normalize_rank_scope(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_MODEL_RANKING_RANK_SCOPE)
        .to_ascii_lowercase()
}

pub fn normalize_snapshot_period(value: Option<&str>) -> String {
    match value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_MODEL_RANKING_SNAPSHOT_PERIOD)
        .to_ascii_lowercase()
        .as_str()
    {
        "hourly" => "hourly",
        "weekly" => "weekly",
        "monthly" => "monthly",
        _ => DEFAULT_MODEL_RANKING_SNAPSHOT_PERIOD,
    }
    .to_owned()
}

pub fn normalize_model_ranking_filter_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

pub fn normalize_model_ranking_search_pattern(value: Option<&str>) -> Option<String> {
    normalize_model_ranking_filter_value(value).map(|value| format!("%{value}%"))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelRankingsCacheInvalidation {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub rank_scope: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingsSnapshot {
    pub source: ModelRankingsSource,
    pub items: Vec<ModelRankingItem>,
    pub history: Vec<ModelRankingHistoryPoint>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingRefreshStatus {
    pub status: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub tenant_id: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub organization_id: i64,
    pub rank_scope: String,
    pub snapshot_date: String,
    pub snapshot_period: String,
    pub window_start: String,
    pub window_end: String,
    pub generated_at: String,
    pub refresh_interval_seconds: i64,
    pub next_refresh_at: String,
    pub cache_max_age_seconds: i64,
    pub generated_count: i64,
    pub source_count: i64,
    pub source_tables: Vec<String>,
    pub latest_job: Option<ModelRankingRefreshJobItem>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingRefreshJobHistoryPage {
    pub items: Vec<ModelRankingRefreshJobItem>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingRefreshJobItem {
    pub id: String,
    pub job_name: String,
    pub status: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub tenant_id: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub organization_id: i64,
    pub rank_scope: String,
    pub snapshot_date: String,
    pub snapshot_period: String,
    pub window_start: String,
    pub window_end: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: i64,
    pub generated_count: i64,
    pub source_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub next_refresh_at: String,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingsSource {
    pub source_label: String,
    pub source_description: String,
    pub observed_at: String,
    pub snapshot_date: String,
    pub snapshot_period: String,
    pub window_start: String,
    pub window_end: String,
    pub generated_at: String,
    pub refresh_interval_seconds: i64,
    pub next_refresh_at: String,
    pub cache_max_age_seconds: i64,
    pub rank_scope: String,
    pub source_tables: Vec<String>,
}

impl Default for ModelRankingsSource {
    fn default() -> Self {
        Self {
            source_label: "Published model ranking snapshot".to_owned(),
            source_description: "Derived from ai_model_rank_snapshot with capability, cost, latency, quality, and routing readiness indicators.".to_owned(),
            observed_at: String::new(),
            snapshot_date: String::new(),
            snapshot_period: "daily".to_owned(),
            window_start: String::new(),
            window_end: String::new(),
            generated_at: String::new(),
            refresh_interval_seconds: 3_600,
            next_refresh_at: String::new(),
            cache_max_age_seconds: 60,
            rank_scope: "commercial-default".to_owned(),
            source_tables: vec!["ai_model_rank_snapshot".to_owned()],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingHistoryPoint {
    pub date: String,
    pub index: i64,
    pub entries: Vec<ModelRankingHistoryEntry>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingHistoryEntry {
    pub model: String,
    pub catalog_key: String,
    pub rank: i64,
    pub volume: i64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankingItem {
    #[serde(skip_serializing)]
    pub observed_at: String,
    pub id: String,
    pub rank: i64,
    pub prev_rank: i64,
    pub name: String,
    pub vendor: String,
    pub vendor_code: String,
    pub modality: String,
    pub base_volume: i64,
    pub cost_indicator: i64,
    pub latency: i64,
    pub context_size: Option<String>,
    pub is_new: bool,
    pub color: String,
    pub win_rate: Option<f64>,
    pub pricing: Option<String>,
    pub license: Option<String>,
    pub strengths: Vec<String>,
    pub requests: i64,
    pub tokens: i64,
    pub cost: f64,
    pub currency: String,
    pub trend_score: Option<f64>,
}

pub trait ModelRankingsReadStore {
    fn load_model_rankings<'a>(
        &'a self,
        query: ModelRankingsQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingsReadFuture<'a>;
}

pub trait ModelRankingRefreshStatusReadStore {
    fn load_model_ranking_refresh_status<'a>(
        &'a self,
        query: ModelRankingRefreshStatusQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshStatusReadFuture<'a> {
        Box::pin(async move {
            let _ = (query, subject);
            Err(DomainError::new(
                "database-backed model ranking refresh status store is not configured",
            ))
        })
    }
}

pub trait ModelRankingRefreshJobHistoryReadStore {
    fn load_model_ranking_refresh_jobs<'a>(
        &'a self,
        query: ModelRankingRefreshJobHistoryQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshJobHistoryReadFuture<'a> {
        Box::pin(async move {
            let _ = (query, subject);
            Err(DomainError::new(
                "database-backed model ranking refresh job history store is not configured",
            ))
        })
    }
}

pub trait ModelRankingsCacheInvalidator {
    fn invalidate_model_rankings_cache(&self, invalidation: ModelRankingsCacheInvalidation) {
        let _ = invalidation;
    }
}

pub trait ModelRankingsReadModelStore:
    ModelRankingsReadStore
    + ModelRankingRefreshStatusReadStore
    + ModelRankingRefreshJobHistoryReadStore
    + ModelRankingsCacheInvalidator
{
}

impl<T> ModelRankingsReadModelStore for T where
    T: ModelRankingsReadStore
        + ModelRankingRefreshStatusReadStore
        + ModelRankingRefreshJobHistoryReadStore
        + ModelRankingsCacheInvalidator
{
}

#[cfg(test)]
mod tests {
    use super::{ModelRankingRefreshJobItem, ModelRankingRefreshStatus};

    #[test]
    fn refresh_status_serializes_scope_ids_as_int64_strings() {
        let status = ModelRankingRefreshStatus {
            tenant_id: i64::MAX,
            organization_id: i64::MAX - 1,
            generated_count: 2,
            latest_job: Some(ModelRankingRefreshJobItem {
                tenant_id: i64::MAX,
                organization_id: i64::MAX - 1,
                duration_ms: 25,
                ..ModelRankingRefreshJobItem::default()
            }),
            ..ModelRankingRefreshStatus::default()
        };

        let payload = serde_json::to_value(status).expect("serialize refresh status");

        assert_eq!("9223372036854775807", payload["tenantId"]);
        assert_eq!("9223372036854775806", payload["organizationId"]);
        assert_eq!(2, payload["generatedCount"]);
        assert_eq!("9223372036854775807", payload["latestJob"]["tenantId"]);
        assert_eq!(25, payload["latestJob"]["durationMs"]);
    }
}
