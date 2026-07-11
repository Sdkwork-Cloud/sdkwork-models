use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_claw_http::TrustedRequestSubject;
use sdkwork_utils_rust::SdkWorkResultCode;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::page_info::{offset_page_info, ApiPageInfo};
use crate::api::response::{finish_success, problem_for};
use crate::api::subject::map_optional_app_user_subject;
use crate::application::{
    ModelRankingRefreshWorker, ModelRankingRefreshWorkerConfig,
    MODEL_RANKING_REFRESH_TRIGGER_MANUAL,
};
use crate::domain::DomainError;
use crate::ports::{
    normalize_model_ranking_filter_value, normalize_rank_scope, ModelRankingHistoryPoint,
    ModelRankingItem, ModelRankingRefreshJobHistoryQuery, ModelRankingRefreshJobHistoryReadFuture,
    ModelRankingRefreshJobHistoryReadStore, ModelRankingRefreshJobItem, ModelRankingRefreshOutcome,
    ModelRankingRefreshStatusQuery, ModelRankingRefreshStatusReadFuture,
    ModelRankingRefreshStatusReadStore, ModelRankingRefreshStore, ModelRankingsCacheInvalidation,
    ModelRankingsCacheInvalidator, ModelRankingsQuery, ModelRankingsReadFuture,
    ModelRankingsReadModelStore, ModelRankingsReadStore, ModelRankingsSnapshot,
    ModelRankingsSource, ModelRankingsSubject,
};

const DEFAULT_RANKING_PAGE_SIZE: i64 = 20;
const MAX_RANKING_LIMIT: i64 = 200;
const DEFAULT_JOB_HISTORY_LIMIT: i64 = 20;
const MAX_JOB_HISTORY_LIMIT: i64 = 200;

#[derive(Clone)]
struct ModelRankingsState {
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
    refresh_store: Option<Arc<dyn ModelRankingRefreshStore + Send + Sync>>,
    manual_refresh_running: Arc<AtomicBool>,
    require_subject: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelRankingsHttpQuery {
    rank_scope: Option<String>,
    vendor_code: Option<String>,
    modality: Option<String>,
    q: Option<String>,
    page_size: Option<i64>,
}

struct UnconfiguredModelRankingsReadStore;

impl ModelRankingsReadStore for UnconfiguredModelRankingsReadStore {
    fn load_model_rankings<'a>(
        &'a self,
        _query: ModelRankingsQuery,
        _subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingsReadFuture<'a> {
        Box::pin(async {
            Err(DomainError::new(
                "database-backed model rankings store is not configured",
            ))
        })
    }
}

impl ModelRankingRefreshStatusReadStore for UnconfiguredModelRankingsReadStore {
    fn load_model_ranking_refresh_status<'a>(
        &'a self,
        _query: ModelRankingRefreshStatusQuery,
        _subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshStatusReadFuture<'a> {
        Box::pin(async {
            Err(DomainError::new(
                "database-backed model ranking refresh status store is not configured",
            ))
        })
    }
}

impl ModelRankingRefreshJobHistoryReadStore for UnconfiguredModelRankingsReadStore {
    fn load_model_ranking_refresh_jobs<'a>(
        &'a self,
        _query: ModelRankingRefreshJobHistoryQuery,
        _subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshJobHistoryReadFuture<'a> {
        Box::pin(async {
            Err(DomainError::new(
                "database-backed model ranking refresh job history store is not configured",
            ))
        })
    }
}

impl ModelRankingsCacheInvalidator for UnconfiguredModelRankingsReadStore {}

pub fn app_model_rankings_router() -> Router {
    model_rankings_router(
        "/app/v3/api/ai/model_rankings",
        Arc::new(UnconfiguredModelRankingsReadStore),
        None,
        false,
        false,
    )
}

pub fn app_model_rankings_router_with_read_store(
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
) -> Router {
    model_rankings_router(
        "/app/v3/api/ai/model_rankings",
        read_store,
        None,
        false,
        false,
    )
}

pub fn admin_model_rankings_router() -> Router {
    model_rankings_router(
        "/backend/v3/api/ai/model_rankings",
        Arc::new(UnconfiguredModelRankingsReadStore),
        None,
        true,
        true,
    )
}

pub fn admin_model_rankings_router_with_read_store(
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
) -> Router {
    model_rankings_router(
        "/backend/v3/api/ai/model_rankings",
        read_store,
        None,
        true,
        true,
    )
}

pub fn admin_model_rankings_router_with_read_store_and_refresh_store(
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
    refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync>,
) -> Router {
    model_rankings_router(
        "/backend/v3/api/ai/model_rankings",
        read_store,
        Some(refresh_store),
        true,
        true,
    )
}

fn model_rankings_router(
    path: &'static str,
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
    refresh_store: Option<Arc<dyn ModelRankingRefreshStore + Send + Sync>>,
    require_subject: bool,
    expose_status: bool,
) -> Router {
    let router = Router::new().route(path, get(fetch_model_rankings));
    let router = if expose_status {
        router
            .route(&format!("{path}/status"), get(fetch_model_ranking_status))
            .route(&format!("{path}/jobs"), get(fetch_model_ranking_jobs))
            .route(
                &format!("{path}/refresh"),
                post(trigger_model_ranking_refresh),
            )
    } else {
        router
    };
    router.with_state(ModelRankingsState {
        read_store,
        refresh_store,
        manual_refresh_running: Arc::new(AtomicBool::new(false)),
        require_subject,
    })
}

async fn trigger_model_ranking_refresh(
    ctx: WebRequestContext,
    State(state): State<ModelRankingsState>,
    trusted: TrustedRequestSubject,
    Json(request): Json<ModelRankingRefreshHttpRequest>,
) -> Response {
    let subject = map_rankings_subject(trusted);
    let Some(refresh_store) = state.refresh_store.clone() else {
        return problem_for(
            &ctx,
            SdkWorkResultCode::ServiceUnavailable,
            "model ranking refresh store is not configured",
        );
    };
    let refresh_guard =
        match ManualRefreshRunningGuard::acquire(Arc::clone(&state.manual_refresh_running)) {
            Ok(guard) => guard,
            Err(()) => {
                return problem_for(
                    &ctx,
                    SdkWorkResultCode::Conflict,
                    "model ranking refresh is already running",
                );
            }
        };

    let result = run_manual_refresh(refresh_store, subject, request).await;
    drop(refresh_guard);
    match result {
        Ok(result) => {
            state
                .read_store
                .invalidate_model_rankings_cache(ModelRankingsCacheInvalidation {
                    tenant_id: subject.tenant_id,
                    organization_id: subject.organization_id,
                    rank_scope: Some(result.rank_scope.clone()),
                });
            finish_success(&ctx, result)
        }
        Err((code, message)) => problem_for(&ctx, code, message),
    }
}

async fn run_manual_refresh(
    refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync>,
    subject: ModelRankingsSubject,
    request: ModelRankingRefreshHttpRequest,
) -> Result<ModelRankingRefreshTriggerResponse, (SdkWorkResultCode, String)> {
    let config = manual_refresh_config(subject, request)?;
    let worker = ModelRankingRefreshWorker::new(refresh_store, config.clone());
    match worker.run_once().await {
        Ok(outcome) => Ok(trigger_response(subject, &config, outcome)),
        Err(error) => Err((
            SdkWorkResultCode::ServiceUnavailable,
            format!("model ranking refresh failed: {error}"),
        )),
    }
}

async fn fetch_model_ranking_jobs(
    ctx: WebRequestContext,
    State(state): State<ModelRankingsState>,
    subject: Option<TrustedRequestSubject>,
    Query(query): Query<ModelRankingJobsHttpQuery>,
) -> Response {
    let subject = match optional_rankings_subject(&ctx, subject, state.require_subject) {
        Ok(subject) => subject,
        Err(response) => return response,
    };

    let limit = query.page_size.unwrap_or(DEFAULT_JOB_HISTORY_LIMIT);
    if !(1..=MAX_JOB_HISTORY_LIMIT).contains(&limit) {
        return problem_for(
            &ctx,
            SdkWorkResultCode::ValidationError,
            format!(
                "model ranking refresh job history page_size must be between 1 and {MAX_JOB_HISTORY_LIMIT}"
            ),
        );
    }

    let query = ModelRankingRefreshJobHistoryQuery {
        rank_scope: Some(normalize_rank_scope(query.rank_scope.as_deref())),
        limit,
    };

    match state
        .read_store
        .load_model_ranking_refresh_jobs(query, subject)
        .await
    {
        Ok(page) => finish_success(&ctx, to_job_history_page_response(page.items, limit)),
        Err(error) => problem_for(
            &ctx,
            SdkWorkResultCode::ServiceUnavailable,
            format!("model ranking refresh job history read model is unavailable: {error}"),
        ),
    }
}

async fn fetch_model_ranking_status(
    ctx: WebRequestContext,
    State(state): State<ModelRankingsState>,
    subject: Option<TrustedRequestSubject>,
    Query(query): Query<ModelRankingStatusHttpQuery>,
) -> Response {
    let subject = match optional_rankings_subject(&ctx, subject, state.require_subject) {
        Ok(subject) => subject,
        Err(response) => return response,
    };

    let query = ModelRankingRefreshStatusQuery {
        rank_scope: Some(normalize_rank_scope(query.rank_scope.as_deref())),
    };

    match state
        .read_store
        .load_model_ranking_refresh_status(query, subject)
        .await
    {
        Ok(status) => finish_success(&ctx, status),
        Err(error) => problem_for(
            &ctx,
            SdkWorkResultCode::ServiceUnavailable,
            format!("model ranking refresh status read model is unavailable: {error}"),
        ),
    }
}

async fn fetch_model_rankings(
    ctx: WebRequestContext,
    State(state): State<ModelRankingsState>,
    subject: Option<TrustedRequestSubject>,
    Query(query): Query<ModelRankingsHttpQuery>,
) -> Response {
    let subject = match optional_rankings_subject(&ctx, subject, state.require_subject) {
        Ok(subject) => subject,
        Err(response) => return response,
    };

    let query = match validate_query(query) {
        Ok(query) => query,
        Err(message) => {
            return problem_for(&ctx, SdkWorkResultCode::ValidationError, message);
        }
    };
    let page_size = query.limit;

    match state.read_store.load_model_rankings(query, subject).await {
        Ok(snapshot) => finish_success(&ctx, to_rankings_page_response(snapshot, page_size)),
        Err(error) => problem_for(
            &ctx,
            SdkWorkResultCode::ServiceUnavailable,
            format!("model rankings read model is unavailable: {error}"),
        ),
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelRankingStatusHttpQuery {
    rank_scope: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelRankingJobsHttpQuery {
    rank_scope: Option<String>,
    page_size: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelRankingRefreshHttpRequest {
    rank_scope: Option<String>,
    snapshot_period: Option<String>,
    page_size: Option<Value>,
    lookback_days: Option<Value>,
    refresh_interval_seconds: Option<Value>,
    cache_max_age_seconds: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelRankingRefreshTriggerResponse {
    triggered: bool,
    status: String,
    tenant_id: String,
    organization_id: String,
    rank_scope: String,
    snapshot_date: String,
    snapshot_period: String,
    window_start: String,
    window_end: String,
    generated_count: String,
    source_count: String,
    refresh_interval_seconds: String,
    cache_max_age_seconds: String,
    next_refresh_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelRankingsPageResponse {
    source: ModelRankingsSource,
    items: Vec<ModelRankingItem>,
    history: Vec<ModelRankingHistoryPoint>,
    page_info: ApiPageInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelRankingRefreshJobHistoryPageResponse {
    items: Vec<ModelRankingRefreshJobItem>,
    page_info: ApiPageInfo,
}

fn map_rankings_subject(trusted: TrustedRequestSubject) -> ModelRankingsSubject {
    ModelRankingsSubject {
        tenant_id: trusted.tenant_id,
        organization_id: trusted.organization_id,
        user_id: trusted.user_id,
    }
}

fn optional_rankings_subject(
    ctx: &WebRequestContext,
    subject: Option<TrustedRequestSubject>,
    require_subject: bool,
) -> Result<Option<ModelRankingsSubject>, Response> {
    map_optional_app_user_subject(ctx, subject, require_subject, map_rankings_subject)
}

fn validate_query(query: ModelRankingsHttpQuery) -> Result<ModelRankingsQuery, String> {
    let limit = query.page_size.unwrap_or(DEFAULT_RANKING_PAGE_SIZE);
    if !(1..=MAX_RANKING_LIMIT).contains(&limit) {
        return Err(format!(
            "model rankings page_size must be between 1 and {MAX_RANKING_LIMIT}"
        ));
    }
    Ok(ModelRankingsQuery {
        rank_scope: Some(normalize_rank_scope(query.rank_scope.as_deref())),
        vendor_code: normalize_model_ranking_filter_value(query.vendor_code.as_deref()),
        modality: normalize_model_ranking_filter_value(query.modality.as_deref()),
        search_query: normalize_model_ranking_filter_value(query.q.as_deref()),
        limit,
    })
}

fn to_rankings_page_response(
    snapshot: ModelRankingsSnapshot,
    page_size: i64,
) -> ModelRankingsPageResponse {
    let total_items = snapshot.items.len() as i64;
    ModelRankingsPageResponse {
        source: snapshot.source,
        items: snapshot.items,
        history: snapshot.history,
        page_info: offset_page_info(1, page_size, total_items),
    }
}

fn to_job_history_page_response(
    items: Vec<ModelRankingRefreshJobItem>,
    page_size: i64,
) -> ModelRankingRefreshJobHistoryPageResponse {
    let total_items = items.len() as i64;
    ModelRankingRefreshJobHistoryPageResponse {
        items,
        page_info: offset_page_info(1, page_size, total_items),
    }
}

fn normalize_query_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

struct ManualRefreshRunningGuard {
    running: Arc<AtomicBool>,
}

impl ManualRefreshRunningGuard {
    fn acquire(running: Arc<AtomicBool>) -> Result<Self, ()> {
        running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| Self { running })
            .map_err(|_| ())
    }
}

impl Drop for ManualRefreshRunningGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn manual_refresh_config(
    subject: ModelRankingsSubject,
    request: ModelRankingRefreshHttpRequest,
) -> Result<ModelRankingRefreshWorkerConfig, (SdkWorkResultCode, String)> {
    let defaults = ModelRankingRefreshWorkerConfig::default();
    let refresh_interval_seconds = validate_optional_range(
        "model ranking refresh interval seconds",
        request.refresh_interval_seconds.as_ref(),
        defaults.interval_millis as i64 / 1_000,
        60,
        86_400,
    )?;

    Ok(ModelRankingRefreshWorkerConfig {
        enabled: true,
        tenant_id: subject.tenant_id,
        organization_id: subject.organization_id,
        rank_scope: validate_rank_scope(request.rank_scope, &defaults.rank_scope)?,
        snapshot_period: validate_snapshot_period(
            request.snapshot_period,
            &defaults.snapshot_period,
        )?,
        limit: validate_optional_range(
            "model ranking refresh limit",
            request.page_size.as_ref(),
            defaults.limit,
            1,
            1_000,
        )?,
        lookback_days: validate_optional_range(
            "model ranking refresh lookback days",
            request.lookback_days.as_ref(),
            defaults.lookback_days,
            1,
            366,
        )?,
        interval_millis: (refresh_interval_seconds as u64) * 1_000,
        cache_max_age_seconds: validate_optional_range(
            "model ranking cache max age seconds",
            request.cache_max_age_seconds.as_ref(),
            defaults.cache_max_age_seconds,
            1,
            86_400,
        )?,
        run_timeout_millis: defaults.run_timeout_millis,
        max_retry_attempts: defaults.max_retry_attempts,
        retry_backoff_millis: defaults.retry_backoff_millis,
        run_on_startup: false,
        alert_after_consecutive_failures: defaults.alert_after_consecutive_failures,
        trigger_type: MODEL_RANKING_REFRESH_TRIGGER_MANUAL,
    })
}

fn validate_rank_scope(
    value: Option<String>,
    fallback: &str,
) -> Result<String, (SdkWorkResultCode, String)> {
    let value = normalize_query_string(value)
        .unwrap_or_else(|| fallback.to_owned())
        .to_ascii_lowercase();
    if value.len() > 80 {
        return bad_refresh_request("model ranking rank scope must be at most 80 characters");
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        Ok(value)
    } else {
        bad_refresh_request(
            "model ranking rank scope may only contain letters, numbers, dash, underscore, dot, or colon",
        )
    }
}

fn validate_snapshot_period(
    value: Option<String>,
    fallback: &str,
) -> Result<String, (SdkWorkResultCode, String)> {
    let value = normalize_query_string(value)
        .unwrap_or_else(|| fallback.to_owned())
        .to_ascii_lowercase();
    match value.as_str() {
        "hourly" | "daily" | "weekly" | "monthly" => Ok(value),
        _ => bad_refresh_request(
            "model ranking snapshot period must be one of hourly, daily, weekly, monthly",
        ),
    }
}

fn validate_optional_range(
    name: &'static str,
    value: Option<&Value>,
    fallback: i64,
    min: i64,
    max: i64,
) -> Result<i64, (SdkWorkResultCode, String)> {
    let value = parse_optional_i64_value(name, value)?.unwrap_or(fallback);
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        bad_refresh_request(format!("{name} must be between {min} and {max}"))
    }
}

fn parse_optional_i64_value(
    name: &'static str,
    value: Option<&Value>,
) -> Result<Option<i64>, (SdkWorkResultCode, String)> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let raw = match value {
        Value::String(value) => value.trim().to_owned(),
        Value::Number(value) => value.to_string(),
        _ => return bad_refresh_request(format!("{name} must be an integer string")),
    };
    if raw.is_empty() {
        return Ok(None);
    }
    raw.parse::<i64>().map(Some).map_err(|_| {
        (
            SdkWorkResultCode::ValidationError,
            format!("{name} must be an integer string"),
        )
    })
}

fn bad_refresh_request<T>(message: impl Into<String>) -> Result<T, (SdkWorkResultCode, String)> {
    Err((SdkWorkResultCode::ValidationError, message.into()))
}

fn trigger_response(
    subject: ModelRankingsSubject,
    config: &ModelRankingRefreshWorkerConfig,
    outcome: ModelRankingRefreshOutcome,
) -> ModelRankingRefreshTriggerResponse {
    ModelRankingRefreshTriggerResponse {
        triggered: true,
        status: if outcome.generated_count > 0 {
            "succeeded"
        } else {
            "empty"
        }
        .to_owned(),
        tenant_id: subject.tenant_id.to_string(),
        organization_id: subject.organization_id.to_string(),
        rank_scope: outcome.rank_scope,
        snapshot_date: outcome.snapshot_date,
        snapshot_period: outcome.snapshot_period,
        window_start: outcome.window_start,
        window_end: outcome.window_end,
        generated_count: outcome.generated_count.to_string(),
        source_count: outcome.source_count.to_string(),
        refresh_interval_seconds: (config.interval_millis / 1_000).to_string(),
        cache_max_age_seconds: config.cache_max_age_seconds.to_string(),
        next_refresh_at: outcome.next_refresh_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_refresh_config_accepts_int64_string_request_fields() {
        let subject = ModelRankingsSubject {
            tenant_id: 100001,
            organization_id: 200002,
            user_id: 300003,
        };
        let config = manual_refresh_config(
            subject,
            ModelRankingRefreshHttpRequest {
                rank_scope: Some("Global".to_owned()),
                snapshot_period: Some("daily".to_owned()),
                page_size: Some(Value::String("50".to_owned())),
                lookback_days: Some(Value::String("14".to_owned())),
                refresh_interval_seconds: Some(Value::String("120".to_owned())),
                cache_max_age_seconds: Some(Value::String("90".to_owned())),
            },
        )
        .expect("string encoded int64 fields should be accepted");

        assert_eq!(config.tenant_id, 100001);
        assert_eq!(config.organization_id, 200002);
        assert_eq!(config.rank_scope, "global");
        assert_eq!(config.snapshot_period, "daily");
        assert_eq!(config.limit, 50);
        assert_eq!(config.lookback_days, 14);
        assert_eq!(config.interval_millis, 120_000);
        assert_eq!(config.cache_max_age_seconds, 90);
    }

    #[test]
    fn manual_refresh_config_rejects_non_integer_request_fields() {
        let subject = ModelRankingsSubject {
            tenant_id: 100001,
            organization_id: 0,
            user_id: 0,
        };
        let error = manual_refresh_config(
            subject,
            ModelRankingRefreshHttpRequest {
                page_size: Some(Value::String("not-an-integer".to_owned())),
                ..ModelRankingRefreshHttpRequest::default()
            },
        )
        .expect_err("invalid int64 strings must be rejected");

        assert_eq!(error.0, SdkWorkResultCode::ValidationError);
        assert!(error.1.contains("model ranking refresh limit"));
    }
}
