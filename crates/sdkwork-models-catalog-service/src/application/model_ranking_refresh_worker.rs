use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tokio::time::{timeout, Duration};

use crate::catalog_time::{
    add_seconds_to_sql_timestamp, current_unix_seconds, date_string_from_unix_seconds,
    iso_timestamp_from_unix, sql_timestamp_from_unix, sql_timestamp_now, start_of_day_unix_seconds,
};
use crate::domain::DomainResult;
use crate::ports::{
    normalize_rank_scope, normalize_scope_ids, normalize_snapshot_period,
    ModelRankingRefreshAuditCommand, ModelRankingRefreshCommand, ModelRankingRefreshOutcome,
    ModelRankingRefreshRunStatus, ModelRankingRefreshStore, DEFAULT_MODEL_RANKING_RANK_SCOPE,
    DEFAULT_MODEL_RANKING_SNAPSHOT_PERIOD,
};

const JOB_NAME: &str = "model_ranking_refresh";
const DEFAULT_RANK_SCOPE: &str = DEFAULT_MODEL_RANKING_RANK_SCOPE;
const DEFAULT_SNAPSHOT_PERIOD: &str = DEFAULT_MODEL_RANKING_SNAPSHOT_PERIOD;
const MIN_LIMIT: i64 = 1;
const DEFAULT_LIMIT: i64 = 200;
const MAX_LIMIT: i64 = 1_000;
const MIN_LOOKBACK_DAYS: i64 = 1;
const DEFAULT_LOOKBACK_DAYS: i64 = 7;
const MAX_LOOKBACK_DAYS: i64 = 366;
const DEFAULT_INTERVAL_MILLIS: u64 = 3_600_000;
const MIN_INTERVAL_MILLIS: u64 = 60_000;
const DEFAULT_CACHE_MAX_AGE_SECONDS: i64 = 60;
const MIN_CACHE_MAX_AGE_SECONDS: i64 = 1;
const DEFAULT_RUN_TIMEOUT_MILLIS: u64 = 300_000;
const MIN_RUN_TIMEOUT_MILLIS: u64 = 5_000;
const MAX_RUN_TIMEOUT_MILLIS: u64 = 3_600_000;
const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 1;
const MAX_RETRY_ATTEMPTS: u32 = 5;
const DEFAULT_RETRY_BACKOFF_MILLIS: u64 = 1_000;
const MIN_RETRY_BACKOFF_MILLIS: u64 = 100;
const MAX_RETRY_BACKOFF_MILLIS: u64 = 60_000;
const DEFAULT_ALERT_AFTER_CONSECUTIVE_FAILURES: i64 = 3;
const MIN_ALERT_AFTER_CONSECUTIVE_FAILURES: i64 = 1;
const MAX_ALERT_AFTER_CONSECUTIVE_FAILURES: i64 = 100;
pub const MODEL_RANKING_REFRESH_TRIGGER_SCHEDULED: i64 = 1;
pub const MODEL_RANKING_REFRESH_TRIGGER_MANUAL: i64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRankingRefreshWorkerConfig {
    pub enabled: bool,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub rank_scope: String,
    pub snapshot_period: String,
    pub limit: i64,
    pub lookback_days: i64,
    pub interval_millis: u64,
    pub cache_max_age_seconds: i64,
    pub run_timeout_millis: u64,
    pub max_retry_attempts: u32,
    pub retry_backoff_millis: u64,
    pub run_on_startup: bool,
    pub alert_after_consecutive_failures: i64,
    pub trigger_type: i64,
}

impl ModelRankingRefreshWorkerConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    pub fn normalized(self) -> Self {
        let (tenant_id, organization_id) =
            normalize_scope_ids(self.tenant_id, self.organization_id);
        Self {
            enabled: self.enabled,
            tenant_id,
            organization_id,
            rank_scope: normalize_rank_scope(Some(&self.rank_scope)),
            snapshot_period: normalize_snapshot_period(Some(&self.snapshot_period)),
            limit: self.limit.clamp(MIN_LIMIT, MAX_LIMIT),
            lookback_days: self
                .lookback_days
                .clamp(MIN_LOOKBACK_DAYS, MAX_LOOKBACK_DAYS),
            interval_millis: self.interval_millis.max(MIN_INTERVAL_MILLIS),
            cache_max_age_seconds: self.cache_max_age_seconds.max(MIN_CACHE_MAX_AGE_SECONDS),
            run_timeout_millis: self
                .run_timeout_millis
                .clamp(MIN_RUN_TIMEOUT_MILLIS, MAX_RUN_TIMEOUT_MILLIS),
            max_retry_attempts: self.max_retry_attempts.min(MAX_RETRY_ATTEMPTS),
            retry_backoff_millis: self
                .retry_backoff_millis
                .clamp(MIN_RETRY_BACKOFF_MILLIS, MAX_RETRY_BACKOFF_MILLIS),
            run_on_startup: self.run_on_startup,
            alert_after_consecutive_failures: self.alert_after_consecutive_failures.clamp(
                MIN_ALERT_AFTER_CONSECUTIVE_FAILURES,
                MAX_ALERT_AFTER_CONSECUTIVE_FAILURES,
            ),
            trigger_type: normalize_trigger_type(self.trigger_type),
        }
    }
}

impl Default for ModelRankingRefreshWorkerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tenant_id: 0,
            organization_id: 0,
            rank_scope: DEFAULT_RANK_SCOPE.to_owned(),
            snapshot_period: DEFAULT_SNAPSHOT_PERIOD.to_owned(),
            limit: DEFAULT_LIMIT,
            lookback_days: DEFAULT_LOOKBACK_DAYS,
            interval_millis: DEFAULT_INTERVAL_MILLIS,
            cache_max_age_seconds: DEFAULT_CACHE_MAX_AGE_SECONDS,
            run_timeout_millis: DEFAULT_RUN_TIMEOUT_MILLIS,
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
            retry_backoff_millis: DEFAULT_RETRY_BACKOFF_MILLIS,
            run_on_startup: true,
            alert_after_consecutive_failures: DEFAULT_ALERT_AFTER_CONSECUTIVE_FAILURES,
            trigger_type: MODEL_RANKING_REFRESH_TRIGGER_SCHEDULED,
        }
    }
}

#[derive(Clone)]
pub struct ModelRankingRefreshWorker {
    store: Arc<dyn ModelRankingRefreshStore + Send + Sync>,
    config: ModelRankingRefreshWorkerConfig,
    running: Arc<AtomicBool>,
    consecutive_failure_count: Arc<AtomicI64>,
}

impl ModelRankingRefreshWorker {
    pub fn new(
        store: Arc<dyn ModelRankingRefreshStore + Send + Sync>,
        config: ModelRankingRefreshWorkerConfig,
    ) -> Self {
        Self {
            store,
            config: config.normalized(),
            running: Arc::new(AtomicBool::new(false)),
            consecutive_failure_count: Arc::new(AtomicI64::new(0)),
        }
    }

    pub fn config(&self) -> &ModelRankingRefreshWorkerConfig {
        &self.config
    }

    pub async fn run_once(&self) -> DomainResult<ModelRankingRefreshOutcome> {
        if !self.config.enabled {
            return Ok(ModelRankingRefreshOutcome::default());
        }
        let run_guard = match ModelRankingRefreshRunGuard::acquire(Arc::clone(&self.running)) {
            Ok(guard) => guard,
            Err(()) => {
                let command = self.build_command();
                self.store
                    .record_model_ranking_refresh_audit(skipped_audit_command(&command))
                    .await?;
                return Ok(ModelRankingRefreshOutcome {
                    rank_scope: command.rank_scope,
                    snapshot_date: command.snapshot_date,
                    snapshot_period: command.snapshot_period,
                    window_start: command.window_start,
                    window_end: command.window_end,
                    next_refresh_at: add_seconds_to_sql_timestamp(
                        &command.requested_at,
                        command.refresh_interval_seconds,
                    ),
                    run_status: ModelRankingRefreshRunStatus::Skipped,
                    ..ModelRankingRefreshOutcome::default()
                });
            }
        };

        let result = self.run_once_locked().await;
        drop(run_guard);
        result
    }

    fn build_command(&self) -> ModelRankingRefreshCommand {
        let now = current_unix_seconds();
        let snapshot_date = date_string_from_unix_seconds(now);
        let window_end = iso_timestamp_from_unix(start_of_day_unix_seconds(now));
        let window_start = iso_timestamp_from_unix(
            start_of_day_unix_seconds(now) - self.config.lookback_days * 86_400,
        );
        let requested_at = sql_timestamp_from_unix(now);

        ModelRankingRefreshCommand {
            tenant_id: self.config.tenant_id,
            organization_id: self.config.organization_id,
            rank_scope: self.config.rank_scope.clone(),
            snapshot_date,
            snapshot_period: self.config.snapshot_period.clone(),
            window_start,
            window_end,
            requested_at: requested_at.clone(),
            limit: self.config.limit,
            refresh_interval_seconds: (self.config.interval_millis / 1_000) as i64,
            cache_max_age_seconds: self.config.cache_max_age_seconds,
            trigger_type: self.config.trigger_type,
        }
    }

    async fn run_once_locked(&self) -> DomainResult<ModelRankingRefreshOutcome> {
        let run_started = Instant::now();
        let command = self.build_command();
        let mut attempt_count = 0_i64;
        let mut last_error = None;

        for attempt_index in 0..=self.config.max_retry_attempts {
            attempt_count = attempt_index as i64 + 1;
            match timeout(
                Duration::from_millis(self.config.run_timeout_millis),
                self.store.refresh_model_rankings(command.clone()),
            )
            .await
            {
                Ok(Ok(mut outcome)) => {
                    outcome.run_status = if outcome.generated_count > 0 {
                        ModelRankingRefreshRunStatus::Succeeded
                    } else {
                        ModelRankingRefreshRunStatus::Empty
                    };
                    self.consecutive_failure_count.store(0, Ordering::SeqCst);
                    self.store
                        .record_model_ranking_refresh_audit(success_audit_command(
                            &command,
                            &outcome,
                            &command.requested_at,
                            run_started,
                            attempt_count,
                        ))
                        .await?;
                    return Ok(outcome);
                }
                Ok(Err(error)) => {
                    last_error = Some(error);
                }
                Err(_) => {
                    last_error = Some(crate::domain::DomainError::new(format!(
                        "model ranking refresh timed out after {} ms",
                        self.config.run_timeout_millis
                    )));
                }
            }

            if attempt_index < self.config.max_retry_attempts {
                tokio::time::sleep(Duration::from_millis(self.config.retry_backoff_millis)).await;
            }
        }

        let error = last_error
            .unwrap_or_else(|| crate::domain::DomainError::new("model ranking refresh failed"));
        let message = error.to_string();
        let consecutive_failure_count = self
            .consecutive_failure_count
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        self.store
            .record_model_ranking_refresh_audit(failure_audit_command(
                &command,
                &command.requested_at,
                run_started,
                message.clone(),
                attempt_count,
                consecutive_failure_count,
                alert_state(
                    consecutive_failure_count,
                    self.config.alert_after_consecutive_failures,
                ),
            ))
            .await?;
        Err(error)
    }
}

struct ModelRankingRefreshRunGuard {
    running: Arc<AtomicBool>,
}

impl ModelRankingRefreshRunGuard {
    fn acquire(running: Arc<AtomicBool>) -> Result<Self, ()> {
        running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| Self { running })
            .map_err(|_| ())
    }
}

impl Drop for ModelRankingRefreshRunGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn success_audit_command(
    command: &ModelRankingRefreshCommand,
    outcome: &ModelRankingRefreshOutcome,
    started_at: &str,
    run_started: Instant,
    attempt_count: i64,
) -> ModelRankingRefreshAuditCommand {
    ModelRankingRefreshAuditCommand {
        job_name: JOB_NAME.to_owned(),
        status: if outcome.generated_count > 0 {
            "succeeded"
        } else {
            "empty"
        }
        .to_owned(),
        tenant_id: command.tenant_id,
        organization_id: command.organization_id,
        rank_scope: outcome.rank_scope.clone(),
        snapshot_date: outcome.snapshot_date.clone(),
        snapshot_period: outcome.snapshot_period.clone(),
        window_start: outcome.window_start.clone(),
        window_end: outcome.window_end.clone(),
        started_at: started_at.to_owned(),
        ended_at: sql_timestamp_now(),
        duration_ms: run_started.elapsed().as_millis().min(i64::MAX as u128) as i64,
        refresh_interval_seconds: command.refresh_interval_seconds,
        cache_max_age_seconds: command.cache_max_age_seconds,
        generated_count: outcome.generated_count,
        source_count: outcome.source_count,
        success_count: outcome.generated_count,
        failure_count: 0,
        next_refresh_at: outcome.next_refresh_at.clone(),
        failure_reason: None,
        trigger_type: command.trigger_type,
        attempt_count,
        retry_count: (attempt_count - 1).max(0),
        consecutive_failure_count: 0,
        alert_recommended: false,
        alert_severity: None,
    }
}

fn failure_audit_command(
    command: &ModelRankingRefreshCommand,
    started_at: &str,
    run_started: Instant,
    failure_reason: String,
    attempt_count: i64,
    consecutive_failure_count: i64,
    alert_state: ModelRankingRefreshAlertState,
) -> ModelRankingRefreshAuditCommand {
    ModelRankingRefreshAuditCommand {
        job_name: JOB_NAME.to_owned(),
        status: "failed".to_owned(),
        tenant_id: command.tenant_id,
        organization_id: command.organization_id,
        rank_scope: command.rank_scope.clone(),
        snapshot_date: command.snapshot_date.clone(),
        snapshot_period: command.snapshot_period.clone(),
        window_start: command.window_start.clone(),
        window_end: command.window_end.clone(),
        started_at: started_at.to_owned(),
        ended_at: sql_timestamp_now(),
        duration_ms: run_started.elapsed().as_millis().min(i64::MAX as u128) as i64,
        refresh_interval_seconds: command.refresh_interval_seconds,
        cache_max_age_seconds: command.cache_max_age_seconds,
        generated_count: 0,
        source_count: 0,
        success_count: 0,
        failure_count: 1,
        next_refresh_at: add_seconds_to_sql_timestamp(
            &command.requested_at,
            command.refresh_interval_seconds,
        ),
        failure_reason: Some(failure_reason),
        trigger_type: command.trigger_type,
        attempt_count,
        retry_count: (attempt_count - 1).max(0),
        consecutive_failure_count,
        alert_recommended: alert_state.recommended,
        alert_severity: alert_state.severity,
    }
}

fn skipped_audit_command(command: &ModelRankingRefreshCommand) -> ModelRankingRefreshAuditCommand {
    let now = sql_timestamp_now();
    ModelRankingRefreshAuditCommand {
        job_name: JOB_NAME.to_owned(),
        status: "skipped".to_owned(),
        tenant_id: command.tenant_id,
        organization_id: command.organization_id,
        rank_scope: command.rank_scope.clone(),
        snapshot_date: command.snapshot_date.clone(),
        snapshot_period: command.snapshot_period.clone(),
        window_start: command.window_start.clone(),
        window_end: command.window_end.clone(),
        started_at: now.clone(),
        ended_at: now,
        duration_ms: 0,
        refresh_interval_seconds: command.refresh_interval_seconds,
        cache_max_age_seconds: command.cache_max_age_seconds,
        generated_count: 0,
        source_count: 0,
        success_count: 0,
        failure_count: 0,
        next_refresh_at: add_seconds_to_sql_timestamp(
            &command.requested_at,
            command.refresh_interval_seconds,
        ),
        failure_reason: Some(
            "model ranking refresh skipped because another run is active".to_owned(),
        ),
        trigger_type: command.trigger_type,
        attempt_count: 0,
        retry_count: 0,
        consecutive_failure_count: 0,
        alert_recommended: false,
        alert_severity: None,
    }
}

struct ModelRankingRefreshAlertState {
    recommended: bool,
    severity: Option<String>,
}

fn alert_state(
    consecutive_failure_count: i64,
    alert_after_consecutive_failures: i64,
) -> ModelRankingRefreshAlertState {
    let recommended = consecutive_failure_count >= alert_after_consecutive_failures.max(1);
    ModelRankingRefreshAlertState {
        recommended,
        severity: if recommended {
            Some(
                if consecutive_failure_count >= alert_after_consecutive_failures.max(1) * 3 {
                    "critical"
                } else {
                    "warning"
                }
                .to_owned(),
            )
        } else {
            None
        },
    }
}

fn normalize_trigger_type(value: i64) -> i64 {
    match value {
        MODEL_RANKING_REFRESH_TRIGGER_MANUAL => MODEL_RANKING_REFRESH_TRIGGER_MANUAL,
        _ => MODEL_RANKING_REFRESH_TRIGGER_SCHEDULED,
    }
}
