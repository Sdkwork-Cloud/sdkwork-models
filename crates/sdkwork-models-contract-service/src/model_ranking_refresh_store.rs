use std::future::Future;
use std::pin::Pin;

use crate::DomainResult;

pub type ModelRankingRefreshFuture<'a> =
    Pin<Box<dyn Future<Output = DomainResult<ModelRankingRefreshOutcome>> + Send + 'a>>;

pub type ModelRankingRefreshAuditFuture<'a> =
    Pin<Box<dyn Future<Output = DomainResult<()>> + Send + 'a>>;

pub trait ModelRankingRefreshStore {
    fn refresh_model_rankings<'a>(
        &'a self,
        command: ModelRankingRefreshCommand,
    ) -> ModelRankingRefreshFuture<'a>;

    fn record_model_ranking_refresh_audit<'a>(
        &'a self,
        command: ModelRankingRefreshAuditCommand,
    ) -> ModelRankingRefreshAuditFuture<'a> {
        Box::pin(async move {
            let _ = command;
            Ok(())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRankingRefreshCommand {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub rank_scope: String,
    pub snapshot_date: String,
    pub snapshot_period: String,
    pub window_start: String,
    pub window_end: String,
    pub requested_at: String,
    pub limit: i64,
    pub refresh_interval_seconds: i64,
    pub cache_max_age_seconds: i64,
    pub trigger_type: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ModelRankingRefreshRunStatus {
    #[default]
    Disabled,
    Succeeded,
    Empty,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelRankingRefreshOutcome {
    pub generated_count: i64,
    pub source_count: i64,
    pub rank_scope: String,
    pub snapshot_date: String,
    pub snapshot_period: String,
    pub window_start: String,
    pub window_end: String,
    pub next_refresh_at: String,
    pub run_status: ModelRankingRefreshRunStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRankingRefreshAuditCommand {
    pub job_name: String,
    pub status: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub rank_scope: String,
    pub snapshot_date: String,
    pub snapshot_period: String,
    pub window_start: String,
    pub window_end: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: i64,
    pub refresh_interval_seconds: i64,
    pub cache_max_age_seconds: i64,
    pub generated_count: i64,
    pub source_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub next_refresh_at: String,
    pub failure_reason: Option<String>,
    pub trigger_type: i64,
    pub attempt_count: i64,
    pub retry_count: i64,
    pub consecutive_failure_count: i64,
    pub alert_recommended: bool,
    pub alert_severity: Option<String>,
}
