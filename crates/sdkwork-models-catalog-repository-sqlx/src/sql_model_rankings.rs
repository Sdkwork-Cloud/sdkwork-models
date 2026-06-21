use serde_json::Value;

use crate::model_modality;
use sdkwork_models_contract_service::{
    ModelRankingHistoryEntry, ModelRankingHistoryPoint, ModelRankingItem,
    ModelRankingRefreshJobItem, ModelRankingRefreshStatus, ModelRankingsSource,
    DEFAULT_MODEL_RANKING_RANK_SCOPE, DEFAULT_MODEL_RANKING_SNAPSHOT_PERIOD,
};

pub const DEFAULT_RANK_SCOPE: &str = DEFAULT_MODEL_RANKING_RANK_SCOPE;
pub const DEFAULT_SNAPSHOT_PERIOD: &str = DEFAULT_MODEL_RANKING_SNAPSHOT_PERIOD;
pub const DEFAULT_REFRESH_INTERVAL_SECONDS: i64 = 3_600;
pub const DEFAULT_CACHE_MAX_AGE_SECONDS: i64 = 60;

pub fn modality_code(value: Option<&str>) -> Option<i64> {
    model_modality::code_from_text(value)
}

pub fn modality_label(value: Option<i64>) -> String {
    model_modality::ranking_label(value).to_owned()
}

pub fn license_label(value: Option<i64>) -> Option<String> {
    match value {
        Some(1) => Some("Open Source".to_owned()),
        Some(2) => Some("Proprietary".to_owned()),
        _ => None,
    }
}

pub fn parse_strengths(raw: Option<String>) -> Vec<String> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return Vec::new();
    };
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RankingSnapshotMetadata {
    pub snapshot_date: String,
    pub snapshot_period: String,
    pub window_start: String,
    pub window_end: String,
    pub generated_at: String,
    pub refresh_interval_seconds: i64,
    pub next_refresh_at: String,
    pub cache_max_age_seconds: i64,
    pub source_tables: Vec<String>,
}

pub fn source_from_items(
    items: &[ModelRankingItem],
    rank_scope: String,
    metadata: RankingSnapshotMetadata,
) -> ModelRankingsSource {
    let observed_at = items
        .first()
        .map(|item| item.observed_at.clone())
        .unwrap_or_else(|| metadata.snapshot_date.clone());
    ModelRankingsSource {
        source_label: "Published model ranking snapshot".to_owned(),
        source_description: "Derived from ai_model_rank_snapshot with capability, cost, latency, quality, and routing readiness indicators.".to_owned(),
        observed_at,
        snapshot_date: metadata.snapshot_date,
        snapshot_period: metadata.snapshot_period,
        window_start: metadata.window_start,
        window_end: metadata.window_end,
        generated_at: metadata.generated_at,
        refresh_interval_seconds: metadata.refresh_interval_seconds,
        next_refresh_at: metadata.next_refresh_at,
        cache_max_age_seconds: metadata.cache_max_age_seconds,
        rank_scope,
        source_tables: if metadata.source_tables.is_empty() {
            default_source_tables()
        } else {
            metadata.source_tables
        },
    }
}

pub fn refresh_status_from_metadata(
    tenant_id: i64,
    organization_id: i64,
    rank_scope: String,
    generated_count: i64,
    source_count: i64,
    metadata: RankingSnapshotMetadata,
) -> ModelRankingRefreshStatus {
    refresh_status_from_metadata_and_latest_job(
        tenant_id,
        organization_id,
        rank_scope,
        generated_count,
        source_count,
        metadata,
        None,
    )
}

pub fn refresh_status_from_metadata_and_latest_job(
    tenant_id: i64,
    organization_id: i64,
    rank_scope: String,
    generated_count: i64,
    source_count: i64,
    metadata: RankingSnapshotMetadata,
    latest_job: Option<ModelRankingRefreshJobItem>,
) -> ModelRankingRefreshStatus {
    ModelRankingRefreshStatus {
        status: if generated_count > 0 {
            "ready"
        } else {
            "empty"
        }
        .to_owned(),
        tenant_id,
        organization_id,
        rank_scope,
        snapshot_date: metadata.snapshot_date,
        snapshot_period: metadata.snapshot_period,
        window_start: metadata.window_start,
        window_end: metadata.window_end,
        generated_at: metadata.generated_at,
        refresh_interval_seconds: metadata.refresh_interval_seconds,
        next_refresh_at: metadata.next_refresh_at,
        cache_max_age_seconds: metadata.cache_max_age_seconds,
        generated_count: generated_count.max(0),
        source_count: source_count.max(generated_count).max(0),
        source_tables: if metadata.source_tables.is_empty() {
            default_source_tables()
        } else {
            metadata.source_tables
        },
        latest_job,
    }
}

pub fn metadata_from_latest_refresh_job(
    job: &ModelRankingRefreshJobItem,
) -> RankingSnapshotMetadata {
    RankingSnapshotMetadata {
        snapshot_date: job.snapshot_date.clone(),
        snapshot_period: job.snapshot_period.clone(),
        window_start: job.window_start.clone(),
        window_end: job.window_end.clone(),
        generated_at: job.ended_at.clone(),
        refresh_interval_seconds: DEFAULT_REFRESH_INTERVAL_SECONDS,
        next_refresh_at: job.next_refresh_at.clone(),
        cache_max_age_seconds: DEFAULT_CACHE_MAX_AGE_SECONDS,
        source_tables: default_source_tables(),
    }
}

pub fn metadata_from_json(
    raw: Option<&str>,
    snapshot_date: String,
    snapshot_period: String,
) -> RankingSnapshotMetadata {
    let value = raw
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .unwrap_or(Value::Null);
    RankingSnapshotMetadata {
        snapshot_date: string_field(&value, "snapshotDate")
            .or_else(|| string_field(&value, "snapshot_date"))
            .unwrap_or(snapshot_date),
        snapshot_period: string_field(&value, "snapshotPeriod")
            .or_else(|| string_field(&value, "snapshot_period"))
            .unwrap_or_else(|| parse_period_cell(snapshot_period.as_str())),
        window_start: string_field(&value, "windowStart")
            .or_else(|| string_field(&value, "window_start"))
            .unwrap_or_default(),
        window_end: string_field(&value, "windowEnd")
            .or_else(|| string_field(&value, "window_end"))
            .unwrap_or_default(),
        generated_at: string_field(&value, "generatedAt")
            .or_else(|| string_field(&value, "generated_at"))
            .unwrap_or_default(),
        refresh_interval_seconds: integer_field(&value, "refreshIntervalSeconds")
            .or_else(|| integer_field(&value, "refresh_interval_seconds"))
            .unwrap_or(DEFAULT_REFRESH_INTERVAL_SECONDS),
        next_refresh_at: string_field(&value, "nextRefreshAt")
            .or_else(|| string_field(&value, "next_refresh_at"))
            .unwrap_or_default(),
        cache_max_age_seconds: integer_field(&value, "cacheMaxAgeSeconds")
            .or_else(|| integer_field(&value, "cache_max_age_seconds"))
            .unwrap_or(DEFAULT_CACHE_MAX_AGE_SECONDS),
        source_tables: string_array_field(&value, "sourceTables")
            .or_else(|| string_array_field(&value, "source_tables"))
            .unwrap_or_else(default_source_tables),
    }
}

pub fn source_rows_from_rank_payload(raw: Option<&str>) -> i64 {
    let Some(raw) = raw else {
        return 0;
    };
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return 0;
    };
    integer_field(&value, "sourceRows")
        .or_else(|| integer_field(&value, "source_rows"))
        .unwrap_or(0)
        .max(0)
}

pub fn refresh_job_item_from_payload(
    id: String,
    job_name: String,
    status: String,
    tenant_id: i64,
    organization_id: i64,
    started_at: String,
    ended_at: String,
    duration_ms: i64,
    success_count: i64,
    failure_count: i64,
    failure_reason: Option<String>,
    payload: Option<&str>,
) -> ModelRankingRefreshJobItem {
    let value = payload
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .unwrap_or(Value::Null);
    ModelRankingRefreshJobItem {
        id,
        job_name,
        status,
        tenant_id,
        organization_id,
        rank_scope: string_field(&value, "rankScope")
            .or_else(|| string_field(&value, "rank_scope"))
            .unwrap_or_else(|| DEFAULT_RANK_SCOPE.to_owned()),
        snapshot_date: string_field(&value, "snapshotDate")
            .or_else(|| string_field(&value, "snapshot_date"))
            .unwrap_or_default(),
        snapshot_period: string_field(&value, "snapshotPeriod")
            .or_else(|| string_field(&value, "snapshot_period"))
            .unwrap_or_else(|| DEFAULT_SNAPSHOT_PERIOD.to_owned()),
        window_start: string_field(&value, "windowStart")
            .or_else(|| string_field(&value, "window_start"))
            .unwrap_or_default(),
        window_end: string_field(&value, "windowEnd")
            .or_else(|| string_field(&value, "window_end"))
            .unwrap_or_default(),
        started_at: normalize_iso_timestamp(&started_at),
        ended_at: normalize_iso_timestamp(&ended_at),
        duration_ms: duration_ms.max(0),
        generated_count: integer_field(&value, "generatedCount")
            .or_else(|| integer_field(&value, "generated_count"))
            .unwrap_or(success_count)
            .max(0),
        source_count: integer_field(&value, "sourceCount")
            .or_else(|| integer_field(&value, "source_count"))
            .unwrap_or(0)
            .max(0),
        success_count: success_count.max(0),
        failure_count: failure_count.max(0),
        next_refresh_at: string_field(&value, "nextRefreshAt")
            .or_else(|| string_field(&value, "next_refresh_at"))
            .unwrap_or_default(),
        failure_reason: failure_reason.filter(|value| !value.trim().is_empty()),
    }
}

pub fn refresh_job_status_label(value: i64, failure_count: i64, success_count: i64) -> String {
    match value {
        2 => "succeeded",
        3 => "failed",
        4 => "empty",
        5 => "skipped",
        _ if failure_count > 0 => "failed",
        _ if success_count > 0 => "succeeded",
        _ => "empty",
    }
    .to_owned()
}

pub fn period_code(value: &str) -> i64 {
    match value.trim().to_ascii_lowercase().as_str() {
        "hourly" => 0,
        "weekly" => 2,
        "monthly" => 3,
        _ => 1,
    }
}

pub fn period_label(value: Option<&str>) -> String {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return DEFAULT_SNAPSHOT_PERIOD.to_owned();
    };
    let lowered = value.to_ascii_lowercase();
    match lowered.as_str() {
        "0" | "hourly" => "hourly",
        "2" | "weekly" => "weekly",
        "3" | "monthly" => "monthly",
        _ => DEFAULT_SNAPSHOT_PERIOD,
    }
    .to_owned()
}

pub fn parse_period_cell(value: &str) -> String {
    period_label(value.trim().split_whitespace().next())
}

pub fn build_history(
    items: Vec<(String, i64, ModelRankingHistoryEntry)>,
) -> Vec<ModelRankingHistoryPoint> {
    let mut history = Vec::<ModelRankingHistoryPoint>::new();
    for (date, index, entry) in items {
        match history.last_mut() {
            Some(point) if point.date == date => {
                if !entry.model.trim().is_empty() {
                    point.entries.push(entry);
                }
            }
            _ => history.push(ModelRankingHistoryPoint {
                date,
                index,
                entries: if entry.model.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![entry]
                },
            }),
        }
    }
    history
}

pub fn normalize_iso_timestamp(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.contains('T') {
        return if trimmed.ends_with('Z') {
            trimmed.to_owned()
        } else {
            format!("{trimmed}Z")
        };
    }
    if trimmed.len() >= 19 {
        return format!("{}T{}Z", &trimmed[0..10], &trimmed[11..19]);
    }
    trimmed.to_owned()
}

pub fn add_seconds_to_timestamp(value: &str, seconds: i64) -> String {
    let Some(base_seconds) = parse_timestamp_to_seconds(value) else {
        return String::new();
    };
    format_iso_timestamp(base_seconds + seconds.max(1))
}

fn default_source_tables() -> Vec<String> {
    vec![
        "ai_usage_fact".to_owned(),
        "ai_request_trace".to_owned(),
        "ai_model_rank_snapshot".to_owned(),
    ]
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn integer_field(value: &Value, key: &str) -> Option<i64> {
    value
        .get(key)
        .and_then(|item| item.as_i64().or_else(|| item.as_str()?.parse::<i64>().ok()))
}

fn string_array_field(value: &Value, key: &str) -> Option<Vec<String>> {
    let items = value.get(key)?.as_array()?;
    let values = items
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
}

fn parse_timestamp_to_seconds(value: &str) -> Option<i64> {
    let timestamp = value.trim().trim_end_matches('Z');
    if timestamp.len() < 19 {
        return None;
    }
    let year = timestamp[0..4].parse::<i64>().ok()?;
    let month = timestamp[5..7].parse::<i64>().ok()?;
    let day = timestamp[8..10].parse::<i64>().ok()?;
    let hour = timestamp[11..13].parse::<i64>().ok()?;
    let minute = timestamp[14..16].parse::<i64>().ok()?;
    let second = timestamp[17..19].parse::<i64>().ok()?;
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn format_iso_timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

pub fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m, d)
}
