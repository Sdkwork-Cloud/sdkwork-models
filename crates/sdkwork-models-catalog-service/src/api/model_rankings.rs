use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_claw_http::TrustedRequestSubject;
use serde::{Deserialize, Serialize};

use crate::api::response::PlusApiResult;
use crate::api::subject::map_optional_app_user_subject;
use crate::application::{
    ModelRankingRefreshWorker, ModelRankingRefreshWorkerConfig,
    MODEL_RANKING_REFRESH_TRIGGER_MANUAL,
};
use crate::domain::DomainError;
use crate::ports::{
    normalize_model_ranking_filter_value, normalize_rank_scope, ModelRankingRefreshJobHistoryQuery,
    ModelRankingRefreshJobHistoryReadFuture, ModelRankingRefreshJobHistoryReadStore,
    ModelRankingRefreshOutcome, ModelRankingRefreshStatusQuery,
    ModelRankingRefreshStatusReadFuture, ModelRankingRefreshStatusReadStore,
    ModelRankingRefreshStore, ModelRankingsCacheInvalidation, ModelRankingsCacheInvalidator,
    ModelRankingsQuery, ModelRankingsReadFuture, ModelRankingsReadModelStore,
    ModelRankingsReadStore, ModelRankingsSubject,
};

const DEFAULT_RANKING_LIMIT: i64 = 50;
const MAX_RANKING_LIMIT: i64 = 200;
const DEFAULT_JOB_HISTORY_LIMIT: i64 = 20;
const MAX_JOB_HISTORY_LIMIT: i64 = 100;

#[derive(Clone)]
struct ModelRankingsState {
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
    refresh_store: Option<Arc<dyn ModelRankingRefreshStore + Send + Sync>>,
    manual_refresh_running: Arc<AtomicBool>,
    require_subject: bool,
}

#[derive(Debug, Deserialize)]
struct ModelRankingsHttpQuery {
    rank_scope: Option<String>,
    vendor_code: Option<String>,
    modality: Option<String>,
    q: Option<String>,
    limit: Option<i64>,
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
    State(state): State<ModelRankingsState>,
    trusted: TrustedRequestSubject,
    Json(request): Json<ModelRankingRefreshHttpRequest>,
) -> Response {
    let subject = map_rankings_subject(trusted);
    let Some(refresh_store) = state.refresh_store.clone() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(PlusApiResult::error(
                "5030",
                "model ranking refresh store is not configured",
            )),
        )
            .into_response();
    };
    let refresh_guard =
        match ManualRefreshRunningGuard::acquire(Arc::clone(&state.manual_refresh_running)) {
            Ok(guard) => guard,
            Err(()) => {
                return (
                    StatusCode::CONFLICT,
                    Json(PlusApiResult::error(
                        "4090",
                        "model ranking refresh is already running",
                    )),
                )
                    .into_response();
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
            Json(PlusApiResult::success(result)).into_response()
        }
        Err((status, code, message)) => {
            (status, Json(PlusApiResult::error(code, message))).into_response()
        }
    }
}

async fn run_manual_refresh(
    refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync>,
    subject: ModelRankingsSubject,
    request: ModelRankingRefreshHttpRequest,
) -> Result<ModelRankingRefreshTriggerResponse, (StatusCode, &'static str, String)> {
    let config = manual_refresh_config(subject, request)?;
    let worker = ModelRankingRefreshWorker::new(refresh_store, config.clone());
    match worker.run_once().await {
        Ok(outcome) => Ok(trigger_response(subject, &config, outcome)),
        Err(error) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "5030",
            format!("model ranking refresh failed: {error}"),
        )),
    }
}

async fn fetch_model_ranking_jobs(
    State(state): State<ModelRankingsState>,
    subject: Option<TrustedRequestSubject>,
    Query(query): Query<ModelRankingJobsHttpQuery>,
) -> Response {
    let subject = match optional_rankings_subject(subject, state.require_subject) {
        Ok(subject) => subject,
        Err(response) => return response,
    };

    let limit = query.limit.unwrap_or(DEFAULT_JOB_HISTORY_LIMIT);
    if !(1..=MAX_JOB_HISTORY_LIMIT).contains(&limit) {
        return (
            StatusCode::BAD_REQUEST,
            Json(PlusApiResult::error(
                "4001",
                format!("model ranking refresh job history limit must be between 1 and {MAX_JOB_HISTORY_LIMIT}"),
            )),
        )
            .into_response();
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
        Ok(page) => Json(PlusApiResult::success(page)).into_response(),
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(PlusApiResult::error(
                "5030",
                format!("model ranking refresh job history read model is unavailable: {error}"),
            )),
        )
            .into_response(),
    }
}

async fn fetch_model_ranking_status(
    State(state): State<ModelRankingsState>,
    subject: Option<TrustedRequestSubject>,
    Query(query): Query<ModelRankingStatusHttpQuery>,
) -> Response {
    let subject = match optional_rankings_subject(subject, state.require_subject) {
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
        Ok(status) => Json(PlusApiResult::success(status)).into_response(),
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(PlusApiResult::error(
                "5030",
                format!("model ranking refresh status read model is unavailable: {error}"),
            )),
        )
            .into_response(),
    }
}

async fn fetch_model_rankings(
    State(state): State<ModelRankingsState>,
    subject: Option<TrustedRequestSubject>,
    Query(query): Query<ModelRankingsHttpQuery>,
) -> Response {
    let subject = match optional_rankings_subject(subject, state.require_subject) {
        Ok(subject) => subject,
        Err(response) => return response,
    };

    let query = match validate_query(query) {
        Ok(query) => query,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PlusApiResult::error("4001", message)),
            )
                .into_response();
        }
    };

    match state.read_store.load_model_rankings(query, subject).await {
        Ok(snapshot) => Json(PlusApiResult::success(snapshot)).into_response(),
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(PlusApiResult::error(
                "5030",
                format!("model rankings read model is unavailable: {error}"),
            )),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct ModelRankingStatusHttpQuery {
    rank_scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModelRankingJobsHttpQuery {
    rank_scope: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelRankingRefreshHttpRequest {
    rank_scope: Option<String>,
    snapshot_period: Option<String>,
    limit: Option<i64>,
    lookback_days: Option<i64>,
    refresh_interval_seconds: Option<i64>,
    cache_max_age_seconds: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelRankingRefreshTriggerResponse {
    triggered: bool,
    status: String,
    tenant_id: i64,
    organization_id: i64,
    rank_scope: String,
    snapshot_date: String,
    snapshot_period: String,
    window_start: String,
    window_end: String,
    generated_count: i64,
    source_count: i64,
    refresh_interval_seconds: i64,
    cache_max_age_seconds: i64,
    next_refresh_at: String,
}

fn map_rankings_subject(trusted: TrustedRequestSubject) -> ModelRankingsSubject {
    ModelRankingsSubject {
        tenant_id: trusted.tenant_id,
        organization_id: trusted.organization_id,
        user_id: trusted.user_id,
    }
}

fn optional_rankings_subject(
    subject: Option<TrustedRequestSubject>,
    require_subject: bool,
) -> Result<Option<ModelRankingsSubject>, Response> {
    map_optional_app_user_subject(subject, require_subject, map_rankings_subject)
}

fn validate_query(query: ModelRankingsHttpQuery) -> Result<ModelRankingsQuery, String> {
    let limit = query.limit.unwrap_or(DEFAULT_RANKING_LIMIT);
    if !(1..=MAX_RANKING_LIMIT).contains(&limit) {
        return Err(format!(
            "model rankings limit must be between 1 and {MAX_RANKING_LIMIT}"
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
) -> Result<ModelRankingRefreshWorkerConfig, (StatusCode, &'static str, String)> {
    let defaults = ModelRankingRefreshWorkerConfig::default();
    let refresh_interval_seconds = validate_optional_range(
        "model ranking refresh interval seconds",
        request.refresh_interval_seconds,
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
            request.limit,
            defaults.limit,
            1,
            1_000,
        )?,
        lookback_days: validate_optional_range(
            "model ranking refresh lookback days",
            request.lookback_days,
            defaults.lookback_days,
            1,
            366,
        )?,
        interval_millis: (refresh_interval_seconds as u64) * 1_000,
        cache_max_age_seconds: validate_optional_range(
            "model ranking cache max age seconds",
            request.cache_max_age_seconds,
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
) -> Result<String, (StatusCode, &'static str, String)> {
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
) -> Result<String, (StatusCode, &'static str, String)> {
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
    value: Option<i64>,
    fallback: i64,
    min: i64,
    max: i64,
) -> Result<i64, (StatusCode, &'static str, String)> {
    let value = value.unwrap_or(fallback);
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        bad_refresh_request(format!("{name} must be between {min} and {max}"))
    }
}

fn bad_refresh_request<T>(
    message: impl Into<String>,
) -> Result<T, (StatusCode, &'static str, String)> {
    Err((StatusCode::BAD_REQUEST, "4001", message.into()))
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
        tenant_id: subject.tenant_id,
        organization_id: subject.organization_id,
        rank_scope: outcome.rank_scope,
        snapshot_date: outcome.snapshot_date,
        snapshot_period: outcome.snapshot_period,
        window_start: outcome.window_start,
        window_end: outcome.window_end,
        generated_count: outcome.generated_count,
        source_count: outcome.source_count,
        refresh_interval_seconds: (config.interval_millis / 1_000) as i64,
        cache_max_age_seconds: config.cache_max_age_seconds,
        next_refresh_at: outcome.next_refresh_at,
    }
}
