use sqlx::{PgPool, Row};

use crate::sql_model_rankings::{
    build_history, license_label, metadata_from_json, metadata_from_latest_refresh_job,
    modality_code, modality_label, parse_period_cell, parse_strengths,
    refresh_job_item_from_payload, refresh_job_status_label, refresh_status_from_metadata,
    refresh_status_from_metadata_and_latest_job, source_from_items, source_rows_from_rank_payload,
    DEFAULT_SNAPSHOT_PERIOD,
};
use sdkwork_models_contract_service::DomainError;
use sdkwork_models_contract_service::{
    normalize_model_ranking_filter_value, normalize_model_ranking_search_pattern,
    normalize_rank_scope, normalize_scope_ids, ModelRankingHistoryEntry, ModelRankingItem,
    ModelRankingRefreshJobHistoryPage, ModelRankingRefreshJobHistoryQuery,
    ModelRankingRefreshJobHistoryReadFuture, ModelRankingRefreshJobHistoryReadStore,
    ModelRankingRefreshStatusQuery, ModelRankingRefreshStatusReadFuture,
    ModelRankingRefreshStatusReadStore, ModelRankingsCacheInvalidator, ModelRankingsQuery,
    ModelRankingsReadFuture, ModelRankingsReadStore, ModelRankingsSnapshot, ModelRankingsSubject,
};

const MODEL_RANKING_REFRESH_JOB_TYPE: i64 = 20;

const LOAD_MODEL_RANKINGS: &str = r#"
WITH selected_snapshot AS (
    SELECT
        tenant_id,
        organization_id,
        snapshot_date,
        snapshot_period,
        COALESCE(rank_scope, 'commercial-default') AS rank_scope
    FROM ai_model_rank_snapshot
    WHERE status = 1
      AND (
          ($1 > 0 AND tenant_id = $1 AND organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0)
          OR (tenant_id = 0 AND organization_id = 0)
      )
      AND lower(COALESCE(rank_scope, 'commercial-default')) = $3
    GROUP BY tenant_id, organization_id, snapshot_date, snapshot_period, COALESCE(rank_scope, 'commercial-default')
    ORDER BY
        CASE
            WHEN $1 > 0 AND tenant_id = $1 AND organization_id = $2 THEN 3
            WHEN $1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0 THEN 2
            WHEN tenant_id = 0 AND organization_id = 0 THEN 1
            ELSE 0
        END DESC,
        snapshot_date DESC NULLS LAST,
        snapshot_period DESC NULLS LAST
    LIMIT 1
),
public_model_catalog AS (
    SELECT DISTINCT m.catalog_key
    FROM ai_model m
    WHERE m.deleted_at IS NULL
      AND m.status = 1
      AND COALESCE(m.release_stage, 1) IN (1, 2)
      AND COALESCE(m.shelf_state, 1) = 1
      AND COALESCE(m.routing_state, 1) = 1
      AND COALESCE(NULLIF(m.catalog_key, ''), '') <> ''
      AND (
          ($1 > 0 AND m.tenant_id = $1 AND m.organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND m.tenant_id = $1 AND m.organization_id = 0)
          OR (m.tenant_id = 0 AND m.organization_id = 0)
      )
)
SELECT
    CAST(COALESCE(r.snapshot_date, CURRENT_DATE) AS TEXT) AS snapshot_date,
    CAST(COALESCE(r.snapshot_period, 1) AS TEXT) AS snapshot_period,
    COALESCE(r.rank_scope, 'commercial-default') AS rank_scope,
    COALESCE(r.rank_no, 0) AS rank_no,
    COALESCE(r.previous_rank_no, r.rank_no, 0) AS previous_rank_no,
    COALESCE(r.catalog_key, '') AS catalog_key,
    COALESCE(r.model, 'unknown') AS model,
    COALESCE(r.vendor_code, '') AS vendor_code,
    COALESCE(NULLIF(r.vendor_name_snapshot, ''), COALESCE(r.vendor_code, 'Unknown')) AS vendor,
    r.modality,
    CAST(COALESCE(r.base_volume, r.request_count, 0) AS TEXT) AS base_volume,
    CASE
        WHEN r.cost_indicator BETWEEN 1 AND 5 THEN r.cost_indicator
        ELSE 3
    END AS cost_indicator,
    COALESCE(r.latency_p50_ms, 0) AS latency_p50_ms,
    NULLIF(r.context_size_text, '') AS context_size_text,
    COALESCE(r.is_new, false) AS is_new,
    COALESCE(NULLIF(r.color_token, ''), '#64748b') AS color_token,
    CAST(r.win_rate AS TEXT) AS win_rate,
    NULLIF(r.pricing_text, '') AS pricing_text,
    r.license_type,
    CAST(r.strengths AS TEXT) AS strengths,
    CAST(COALESCE(r.request_count, 0) AS TEXT) AS request_count,
    CAST(COALESCE(r.token_count, 0) AS TEXT) AS token_count,
    CAST(COALESCE(r.cost_amount, 0) AS TEXT) AS cost_amount,
    COALESCE(r.currency, 'USD') AS currency,
    CAST(r.trend_score AS TEXT) AS trend_score,
    CAST(COALESCE(r.metadata, '{}'::jsonb) AS TEXT) AS metadata
FROM ai_model_rank_snapshot r
JOIN selected_snapshot s
  ON r.tenant_id = s.tenant_id
 AND r.organization_id = s.organization_id
 AND r.snapshot_date = s.snapshot_date
 AND r.snapshot_period = s.snapshot_period
 AND COALESCE(r.rank_scope, 'commercial-default') = s.rank_scope
JOIN public_model_catalog visible_model
  ON visible_model.catalog_key = NULLIF(r.catalog_key, '')
WHERE r.status = 1
  AND ($4::text IS NULL OR lower(COALESCE(r.vendor_code, '')) = $4)
  AND ($5::int8 IS NULL OR r.modality = $5)
  AND ($6::text IS NULL OR lower(COALESCE(r.model, '') || ' ' || COALESCE(r.vendor_name_snapshot, '') || ' ' || COALESCE(r.vendor_code, '')) LIKE $6)
ORDER BY r.rank_no ASC NULLS LAST, r.id DESC
LIMIT $7
"#;

const LOAD_MODEL_RANKING_SOURCE: &str = r#"
WITH selected_snapshot AS (
    SELECT
        tenant_id,
        organization_id,
        snapshot_date,
        snapshot_period,
        COALESCE(rank_scope, 'commercial-default') AS rank_scope
    FROM ai_model_rank_snapshot
    WHERE status = 1
      AND (
          ($1 > 0 AND tenant_id = $1 AND organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0)
          OR (tenant_id = 0 AND organization_id = 0)
      )
      AND lower(COALESCE(rank_scope, 'commercial-default')) = $3
    GROUP BY tenant_id, organization_id, snapshot_date, snapshot_period, COALESCE(rank_scope, 'commercial-default')
    ORDER BY
        CASE
            WHEN $1 > 0 AND tenant_id = $1 AND organization_id = $2 THEN 3
            WHEN $1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0 THEN 2
            WHEN tenant_id = 0 AND organization_id = 0 THEN 1
            ELSE 0
        END DESC,
        snapshot_date DESC NULLS LAST,
        snapshot_period DESC NULLS LAST
    LIMIT 1
),
public_model_catalog AS (
    SELECT DISTINCT m.catalog_key
    FROM ai_model m
    WHERE m.deleted_at IS NULL
      AND m.status = 1
      AND COALESCE(m.release_stage, 1) IN (1, 2)
      AND COALESCE(m.shelf_state, 1) = 1
      AND COALESCE(m.routing_state, 1) = 1
      AND COALESCE(NULLIF(m.catalog_key, ''), '') <> ''
      AND (
          ($1 > 0 AND m.tenant_id = $1 AND m.organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND m.tenant_id = $1 AND m.organization_id = 0)
          OR (m.tenant_id = 0 AND m.organization_id = 0)
      )
)
SELECT
    CAST(COALESCE(r.snapshot_date, CURRENT_DATE) AS TEXT) AS snapshot_date,
    CAST(COALESCE(r.snapshot_period, 1) AS TEXT) AS snapshot_period,
    COALESCE(r.rank_scope, 'commercial-default') AS rank_scope,
    CAST(COALESCE(r.metadata, '{}'::jsonb) AS TEXT) AS metadata
FROM ai_model_rank_snapshot r
JOIN selected_snapshot s
  ON r.tenant_id = s.tenant_id
 AND r.organization_id = s.organization_id
 AND r.snapshot_date = s.snapshot_date
 AND r.snapshot_period = s.snapshot_period
 AND COALESCE(r.rank_scope, 'commercial-default') = s.rank_scope
JOIN public_model_catalog visible_model
  ON visible_model.catalog_key = NULLIF(r.catalog_key, '')
WHERE r.status = 1
ORDER BY r.rank_no ASC NULLS LAST, r.id ASC
LIMIT 1
"#;

const LOAD_MODEL_RANKING_HISTORY: &str = r#"
WITH selected_scope AS (
    SELECT
        tenant_id,
        organization_id,
        COALESCE(rank_scope, 'commercial-default') AS rank_scope
    FROM ai_model_rank_snapshot
    WHERE status = 1
      AND (
          ($1 > 0 AND tenant_id = $1 AND organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0)
          OR (tenant_id = 0 AND organization_id = 0)
      )
      AND lower(COALESCE(rank_scope, 'commercial-default')) = $3
    GROUP BY tenant_id, organization_id, COALESCE(rank_scope, 'commercial-default')
    ORDER BY
        CASE
            WHEN $1 > 0 AND tenant_id = $1 AND organization_id = $2 THEN 3
            WHEN $1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0 THEN 2
            WHEN tenant_id = 0 AND organization_id = 0 THEN 1
            ELSE 0
        END DESC,
        MAX(snapshot_date) DESC NULLS LAST,
        MAX(snapshot_period) DESC NULLS LAST
    LIMIT 1
),
latest_snapshot AS (
    SELECT
        r.tenant_id,
        r.organization_id,
        r.snapshot_date,
        r.snapshot_period,
        COALESCE(r.rank_scope, 'commercial-default') AS rank_scope
    FROM ai_model_rank_snapshot r
    JOIN selected_scope s
      ON r.tenant_id = s.tenant_id
     AND r.organization_id = s.organization_id
     AND COALESCE(r.rank_scope, 'commercial-default') = s.rank_scope
    WHERE r.status = 1
    GROUP BY r.tenant_id, r.organization_id, r.snapshot_date, r.snapshot_period, COALESCE(r.rank_scope, 'commercial-default')
    ORDER BY r.snapshot_date DESC NULLS LAST, r.snapshot_period DESC NULLS LAST
    LIMIT 1
),
public_model_catalog AS (
    SELECT DISTINCT m.catalog_key
    FROM ai_model m
    WHERE m.deleted_at IS NULL
      AND m.status = 1
      AND COALESCE(m.release_stage, 1) IN (1, 2)
      AND COALESCE(m.shelf_state, 1) = 1
      AND COALESCE(m.routing_state, 1) = 1
      AND COALESCE(NULLIF(m.catalog_key, ''), '') <> ''
      AND (
          ($1 > 0 AND m.tenant_id = $1 AND m.organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND m.tenant_id = $1 AND m.organization_id = 0)
          OR (m.tenant_id = 0 AND m.organization_id = 0)
      )
),
selected_catalog_keys AS (
    SELECT
        NULLIF(r.catalog_key, '') AS catalog_key
    FROM ai_model_rank_snapshot r
    JOIN latest_snapshot s
      ON r.tenant_id = s.tenant_id
     AND r.organization_id = s.organization_id
     AND r.snapshot_date = s.snapshot_date
     AND r.snapshot_period = s.snapshot_period
     AND COALESCE(r.rank_scope, 'commercial-default') = s.rank_scope
    JOIN public_model_catalog visible_model
      ON visible_model.catalog_key = NULLIF(r.catalog_key, '')
    WHERE r.status = 1
      AND ($4::text IS NULL OR lower(COALESCE(r.vendor_code, '')) = $4)
      AND ($5::int8 IS NULL OR r.modality = $5)
      AND ($6::text IS NULL OR lower(COALESCE(r.model, '') || ' ' || COALESCE(r.vendor_name_snapshot, '') || ' ' || COALESCE(r.vendor_code, '')) LIKE $6)
    GROUP BY NULLIF(r.catalog_key, '')
    ORDER BY MIN(COALESCE(r.rank_no, 2147483647)) ASC, MAX(r.id) DESC
    LIMIT $7
),
selected_snapshots AS (
    SELECT
        tenant_id,
        organization_id,
        snapshot_date,
        snapshot_period,
        rank_scope
    FROM latest_snapshot
    ORDER BY snapshot_date ASC NULLS LAST, snapshot_period ASC NULLS LAST
),
ranked AS (
    SELECT
        CAST(s.snapshot_date AS TEXT) AS snapshot_date,
        COALESCE(r.catalog_key, '') AS catalog_key,
        COALESCE(r.model, '') AS model,
        COALESCE(r.rank_no, 0) AS rank_no,
        CAST(COALESCE(r.base_volume, r.request_count, 0) AS TEXT) AS volume,
        COALESCE(NULLIF(r.color_token, ''), '#64748b') AS color_token
    FROM selected_snapshots s
    JOIN selected_catalog_keys k
      ON true
    LEFT JOIN ai_model_rank_snapshot r
      ON r.tenant_id = s.tenant_id
     AND r.organization_id = s.organization_id
     AND r.snapshot_date = s.snapshot_date
     AND r.snapshot_period = s.snapshot_period
     AND COALESCE(r.rank_scope, 'commercial-default') = s.rank_scope
     AND r.status = 1
     AND NULLIF(r.catalog_key, '') = k.catalog_key
)
SELECT
    snapshot_date,
    CAST(DENSE_RANK() OVER (ORDER BY snapshot_date ASC) - 1 AS TEXT) AS history_index,
    catalog_key,
    model,
    CAST(rank_no AS TEXT) AS rank_no,
    volume,
    color_token
FROM ranked
ORDER BY snapshot_date ASC, rank_no ASC, model ASC
"#;

const LOAD_MODEL_RANKING_REFRESH_STATUS: &str = r#"
WITH selected_snapshot AS (
    SELECT
        tenant_id,
        organization_id,
        snapshot_date,
        snapshot_period,
        COALESCE(rank_scope, 'commercial-default') AS rank_scope
    FROM ai_model_rank_snapshot
    WHERE status = 1
      AND (
          ($1 > 0 AND tenant_id = $1 AND organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0)
          OR (tenant_id = 0 AND organization_id = 0)
      )
      AND lower(COALESCE(rank_scope, 'commercial-default')) = $3
    GROUP BY tenant_id, organization_id, snapshot_date, snapshot_period, COALESCE(rank_scope, 'commercial-default')
    ORDER BY
        CASE
            WHEN $1 > 0 AND tenant_id = $1 AND organization_id = $2 THEN 3
            WHEN $1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0 THEN 2
            WHEN tenant_id = 0 AND organization_id = 0 THEN 1
            ELSE 0
        END DESC,
        snapshot_date DESC NULLS LAST,
        snapshot_period DESC NULLS LAST
    LIMIT 1
),
public_model_catalog AS (
    SELECT DISTINCT m.catalog_key
    FROM ai_model m
    WHERE m.deleted_at IS NULL
      AND m.status = 1
      AND COALESCE(m.release_stage, 1) IN (1, 2)
      AND COALESCE(m.shelf_state, 1) = 1
      AND COALESCE(m.routing_state, 1) = 1
      AND COALESCE(NULLIF(m.catalog_key, ''), '') <> ''
      AND (
          ($1 > 0 AND m.tenant_id = $1 AND m.organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND m.tenant_id = $1 AND m.organization_id = 0)
          OR (m.tenant_id = 0 AND m.organization_id = 0)
      )
)
SELECT
    CAST(r.tenant_id AS TEXT) AS tenant_id,
    CAST(r.organization_id AS TEXT) AS organization_id,
    CAST(COALESCE(r.snapshot_date, CURRENT_DATE) AS TEXT) AS snapshot_date,
    CAST(COALESCE(r.snapshot_period, 1) AS TEXT) AS snapshot_period,
    COALESCE(r.rank_scope, 'commercial-default') AS rank_scope,
    CAST(COALESCE(r.metadata, '{}'::jsonb) AS TEXT) AS metadata,
    CAST(COALESCE(r.rank_payload, '{}'::jsonb) AS TEXT) AS rank_payload
FROM ai_model_rank_snapshot r
JOIN selected_snapshot s
  ON r.tenant_id = s.tenant_id
 AND r.organization_id = s.organization_id
 AND r.snapshot_date = s.snapshot_date
 AND r.snapshot_period = s.snapshot_period
 AND COALESCE(r.rank_scope, 'commercial-default') = s.rank_scope
JOIN public_model_catalog visible_model
  ON visible_model.catalog_key = NULLIF(r.catalog_key, '')
WHERE r.status = 1
ORDER BY r.rank_no ASC NULLS LAST, r.id ASC
"#;

const LOAD_MODEL_RANKING_REFRESH_JOBS: &str = r#"
WITH selected_snapshot_scope AS (
    SELECT
        tenant_id,
        organization_id
    FROM ai_model_rank_snapshot
    WHERE status = 1
      AND (
          ($1 > 0 AND tenant_id = $1 AND organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0)
          OR (tenant_id = 0 AND organization_id = 0)
      )
      AND lower(COALESCE(rank_scope, 'commercial-default')) = $3
    GROUP BY tenant_id, organization_id
    ORDER BY
        CASE
            WHEN $1 > 0 AND tenant_id = $1 AND organization_id = $2 THEN 3
            WHEN $1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0 THEN 2
            WHEN tenant_id = 0 AND organization_id = 0 THEN 1
            ELSE 0
        END DESC,
        MAX(snapshot_date) DESC NULLS LAST,
        MAX(snapshot_period) DESC NULLS LAST
    LIMIT 1
),
selected_fallback_job_scope AS (
    SELECT
        tenant_id,
        organization_id
    FROM ops_job_execution
    WHERE status = 1
      AND COALESCE(job_name, '') = 'model_ranking_refresh'
      AND job_type = $4
      AND (
          ($1 > 0 AND tenant_id = $1 AND organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0)
          OR (tenant_id = 0 AND organization_id = 0)
      )
      AND lower(COALESCE(payload->>'rankScope', payload->>'rank_scope', 'commercial-default')) = $3
    GROUP BY tenant_id, organization_id
    ORDER BY
        CASE
            WHEN $1 > 0 AND tenant_id = $1 AND organization_id = $2 THEN 3
            WHEN $1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0 THEN 2
            WHEN tenant_id = 0 AND organization_id = 0 THEN 1
            ELSE 0
        END DESC,
        MAX(started_at) DESC NULLS LAST,
        MAX(id) DESC
    LIMIT 1
),
selected_job_scope AS (
    SELECT tenant_id, organization_id
    FROM selected_snapshot_scope
    UNION ALL
    SELECT tenant_id, organization_id
    FROM selected_fallback_job_scope
    WHERE NOT EXISTS (SELECT 1 FROM selected_snapshot_scope)
    LIMIT 1
)
SELECT
    COALESCE(j.uuid, CAST(j.id AS TEXT)) AS id,
    COALESCE(j.job_name, 'model_ranking_refresh') AS job_name,
    CAST(j.tenant_id AS TEXT) AS tenant_id,
    CAST(j.organization_id AS TEXT) AS organization_id,
    CAST(COALESCE(j.started_at, CURRENT_TIMESTAMP) AS TEXT) AS started_at,
    CAST(COALESCE(j.ended_at, CURRENT_TIMESTAMP) AS TEXT) AS ended_at,
    CAST(COALESCE(j.duration_ms, 0) AS TEXT) AS duration_ms,
    CAST(COALESCE(j.execution_status, 0) AS TEXT) AS execution_status,
    CAST(COALESCE(j.success_count, 0) AS TEXT) AS success_count,
    CAST(COALESCE(j.failure_count, 0) AS TEXT) AS failure_count,
    NULLIF(j.failure_reason, '') AS failure_reason,
    CAST(COALESCE(j.payload, '{}'::jsonb) AS TEXT) AS payload
FROM ops_job_execution j
JOIN selected_job_scope s
  ON j.tenant_id = s.tenant_id
 AND j.organization_id = s.organization_id
WHERE j.status = 1
  AND COALESCE(j.job_name, '') = 'model_ranking_refresh'
  AND j.job_type = $4
  AND lower(COALESCE(j.payload->>'rankScope', j.payload->>'rank_scope', 'commercial-default')) = $3
ORDER BY j.started_at DESC NULLS LAST, j.id DESC
LIMIT $5
"#;

const LOAD_LATEST_MODEL_RANKING_REFRESH_JOB: &str = r#"
WITH selected_job_scope AS (
    SELECT
        tenant_id,
        organization_id
    FROM ops_job_execution
    WHERE status = 1
      AND COALESCE(job_name, '') = 'model_ranking_refresh'
      AND job_type = $4
      AND (
          ($1 > 0 AND tenant_id = $1 AND organization_id = $2)
          OR ($1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0)
          OR (tenant_id = 0 AND organization_id = 0)
      )
      AND lower(COALESCE(payload->>'rankScope', payload->>'rank_scope', 'commercial-default')) = $3
    GROUP BY tenant_id, organization_id
    ORDER BY
        CASE
            WHEN $1 > 0 AND tenant_id = $1 AND organization_id = $2 THEN 3
            WHEN $1 > 0 AND $2 > 0 AND tenant_id = $1 AND organization_id = 0 THEN 2
            WHEN tenant_id = 0 AND organization_id = 0 THEN 1
            ELSE 0
        END DESC,
        MAX(started_at) DESC NULLS LAST,
        MAX(id) DESC
    LIMIT 1
)
SELECT
    COALESCE(j.uuid, CAST(j.id AS TEXT)) AS id,
    COALESCE(j.job_name, 'model_ranking_refresh') AS job_name,
    CAST(j.tenant_id AS TEXT) AS tenant_id,
    CAST(j.organization_id AS TEXT) AS organization_id,
    CAST(COALESCE(j.started_at, CURRENT_TIMESTAMP) AS TEXT) AS started_at,
    CAST(COALESCE(j.ended_at, CURRENT_TIMESTAMP) AS TEXT) AS ended_at,
    CAST(COALESCE(j.duration_ms, 0) AS TEXT) AS duration_ms,
    CAST(COALESCE(j.execution_status, 0) AS TEXT) AS execution_status,
    CAST(COALESCE(j.success_count, 0) AS TEXT) AS success_count,
    CAST(COALESCE(j.failure_count, 0) AS TEXT) AS failure_count,
    NULLIF(j.failure_reason, '') AS failure_reason,
    CAST(COALESCE(j.payload, '{}'::jsonb) AS TEXT) AS payload
FROM ops_job_execution j
JOIN selected_job_scope s
  ON j.tenant_id = s.tenant_id
 AND j.organization_id = s.organization_id
WHERE j.status = 1
  AND COALESCE(j.job_name, '') = 'model_ranking_refresh'
  AND j.job_type = $4
  AND lower(COALESCE(j.payload->>'rankScope', j.payload->>'rank_scope', 'commercial-default')) = $3
ORDER BY j.started_at DESC NULLS LAST, j.id DESC
LIMIT 1
"#;

const LOAD_LATEST_MODEL_RANKING_REFRESH_JOB_FOR_SCOPE: &str = r#"
SELECT
    COALESCE(uuid, CAST(id AS TEXT)) AS id,
    COALESCE(job_name, 'model_ranking_refresh') AS job_name,
    CAST(tenant_id AS TEXT) AS tenant_id,
    CAST(organization_id AS TEXT) AS organization_id,
    CAST(COALESCE(started_at, CURRENT_TIMESTAMP) AS TEXT) AS started_at,
    CAST(COALESCE(ended_at, CURRENT_TIMESTAMP) AS TEXT) AS ended_at,
    CAST(COALESCE(duration_ms, 0) AS TEXT) AS duration_ms,
    CAST(COALESCE(execution_status, 0) AS TEXT) AS execution_status,
    CAST(COALESCE(success_count, 0) AS TEXT) AS success_count,
    CAST(COALESCE(failure_count, 0) AS TEXT) AS failure_count,
    NULLIF(failure_reason, '') AS failure_reason,
    CAST(COALESCE(payload, '{}'::jsonb) AS TEXT) AS payload
FROM ops_job_execution
WHERE status = 1
  AND COALESCE(job_name, '') = 'model_ranking_refresh'
  AND job_type = $4
  AND tenant_id = $1
  AND organization_id = $2
  AND lower(COALESCE(payload->>'rankScope', payload->>'rank_scope', 'commercial-default')) = $3
ORDER BY started_at DESC NULLS LAST, id DESC
LIMIT 1
"#;

pub struct PostgresModelRankingsReadStore {
    pool: PgPool,
}

impl PostgresModelRankingsReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ModelRankingsReadStore for PostgresModelRankingsReadStore {
    fn load_model_rankings<'a>(
        &'a self,
        query: ModelRankingsQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingsReadFuture<'a> {
        Box::pin(async move {
            let subject = normalize_subject(subject.unwrap_or(ModelRankingsSubject {
                tenant_id: 0,
                organization_id: 0,
                user_id: 0,
            }));
            let modality = modality_code(query.modality.as_deref());
            let vendor_code = normalize_model_ranking_filter_value(query.vendor_code.as_deref());
            let search = normalize_model_ranking_search_pattern(query.search_query.as_deref());
            let rank_scope = normalize_rank_scope(query.rank_scope.as_deref());
            let rows = sqlx::query(LOAD_MODEL_RANKINGS)
                .bind(subject.tenant_id)
                .bind(subject.organization_id)
                .bind(rank_scope.as_str())
                .bind(vendor_code.as_deref())
                .bind(modality)
                .bind(search.as_deref())
                .bind(query.limit)
                .fetch_all(&self.pool)
                .await
                .map_err(sql_error)?;

            let metadata = if let Some(first) = rows.first() {
                snapshot_metadata(first)
            } else {
                load_source_metadata(&self.pool, subject, rank_scope.as_str())
                    .await?
                    .unwrap_or_default()
            };
            let items: Vec<ModelRankingItem> = rows
                .into_iter()
                .enumerate()
                .map(|(index, row)| ranking_item_from_row(index, &row))
                .collect();
            let history = load_history(
                &self.pool,
                subject,
                rank_scope.as_str(),
                vendor_code.as_deref(),
                modality,
                search.as_deref(),
                query.limit,
            )
            .await?;

            Ok(ModelRankingsSnapshot {
                source: source_from_items(&items, rank_scope, metadata),
                items,
                history,
            })
        })
    }
}

impl ModelRankingRefreshStatusReadStore for PostgresModelRankingsReadStore {
    fn load_model_ranking_refresh_status<'a>(
        &'a self,
        query: ModelRankingRefreshStatusQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshStatusReadFuture<'a> {
        Box::pin(async move {
            let subject = normalize_subject(subject.unwrap_or(ModelRankingsSubject {
                tenant_id: 0,
                organization_id: 0,
                user_id: 0,
            }));
            let rank_scope = normalize_rank_scope(query.rank_scope.as_deref());
            let rows = sqlx::query(LOAD_MODEL_RANKING_REFRESH_STATUS)
                .bind(subject.tenant_id)
                .bind(subject.organization_id)
                .bind(rank_scope.as_str())
                .fetch_all(&self.pool)
                .await
                .map_err(sql_error)?;

            let Some(first) = rows.first() else {
                let latest_job =
                    load_latest_refresh_job(&self.pool, subject, rank_scope.as_str()).await?;
                if let Some(latest_job) = latest_job {
                    let job_tenant_id = latest_job.tenant_id;
                    let job_organization_id = latest_job.organization_id;
                    return Ok(refresh_status_from_metadata_and_latest_job(
                        job_tenant_id,
                        job_organization_id,
                        rank_scope,
                        latest_job.generated_count,
                        latest_job.source_count,
                        metadata_from_latest_refresh_job(&latest_job),
                        Some(latest_job),
                    ));
                }
                return Ok(refresh_status_from_metadata(
                    subject.tenant_id,
                    subject.organization_id,
                    rank_scope,
                    0,
                    0,
                    metadata_from_json(None, String::new(), DEFAULT_SNAPSHOT_PERIOD.to_owned()),
                ));
            };
            let metadata = snapshot_metadata(first);
            let snapshot_tenant_id = integer_cell(first, "tenant_id");
            let snapshot_organization_id = integer_cell(first, "organization_id");
            let snapshot_rank_scope = string_cell(first, "rank_scope");
            let latest_job = load_latest_refresh_job_for_scope(
                &self.pool,
                snapshot_tenant_id,
                snapshot_organization_id,
                snapshot_rank_scope.as_str(),
            )
            .await?;
            let source_count = rows
                .iter()
                .map(|row| {
                    source_rows_from_rank_payload(
                        optional_string_cell(row, "rank_payload").as_deref(),
                    )
                })
                .sum::<i64>();

            Ok(refresh_status_from_metadata_and_latest_job(
                snapshot_tenant_id,
                snapshot_organization_id,
                snapshot_rank_scope,
                rows.len() as i64,
                source_count,
                metadata,
                latest_job,
            ))
        })
    }
}

impl ModelRankingRefreshJobHistoryReadStore for PostgresModelRankingsReadStore {
    fn load_model_ranking_refresh_jobs<'a>(
        &'a self,
        query: ModelRankingRefreshJobHistoryQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshJobHistoryReadFuture<'a> {
        Box::pin(async move {
            let subject = normalize_subject(subject.unwrap_or(ModelRankingsSubject {
                tenant_id: 0,
                organization_id: 0,
                user_id: 0,
            }));
            let rank_scope = normalize_rank_scope(query.rank_scope.as_deref());
            let rows = sqlx::query(LOAD_MODEL_RANKING_REFRESH_JOBS)
                .bind(subject.tenant_id)
                .bind(subject.organization_id)
                .bind(rank_scope.as_str())
                .bind(MODEL_RANKING_REFRESH_JOB_TYPE)
                .bind(query.limit)
                .fetch_all(&self.pool)
                .await
                .map_err(sql_error)?;

            Ok(ModelRankingRefreshJobHistoryPage {
                items: rows.iter().map(refresh_job_item_from_row).collect(),
            })
        })
    }
}

impl ModelRankingsCacheInvalidator for PostgresModelRankingsReadStore {}

async fn load_history(
    pool: &PgPool,
    subject: ModelRankingsSubject,
    rank_scope: &str,
    vendor_code: Option<&str>,
    modality: Option<i64>,
    search: Option<&str>,
    limit: i64,
) -> Result<Vec<sdkwork_models_contract_service::ModelRankingHistoryPoint>, DomainError> {
    let rows = sqlx::query(LOAD_MODEL_RANKING_HISTORY)
        .bind(subject.tenant_id)
        .bind(subject.organization_id)
        .bind(rank_scope)
        .bind(vendor_code)
        .bind(modality)
        .bind(search)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(sql_error)?;

    Ok(build_history(
        rows.into_iter()
            .map(|row| {
                (
                    string_cell(&row, "snapshot_date"),
                    integer_cell(&row, "history_index"),
                    ModelRankingHistoryEntry {
                        model: string_cell(&row, "model"),
                        catalog_key: string_cell(&row, "catalog_key"),
                        rank: integer_cell(&row, "rank_no"),
                        volume: integer_cell(&row, "volume"),
                        color: string_cell(&row, "color_token"),
                    },
                )
            })
            .collect(),
    ))
}

async fn load_source_metadata(
    pool: &PgPool,
    subject: ModelRankingsSubject,
    rank_scope: &str,
) -> Result<Option<crate::sql_model_rankings::RankingSnapshotMetadata>, DomainError> {
    let row = sqlx::query(LOAD_MODEL_RANKING_SOURCE)
        .bind(subject.tenant_id)
        .bind(subject.organization_id)
        .bind(rank_scope)
        .fetch_optional(pool)
        .await
        .map_err(sql_error)?;

    Ok(row.as_ref().map(snapshot_metadata))
}

async fn load_latest_refresh_job(
    pool: &PgPool,
    subject: ModelRankingsSubject,
    rank_scope: &str,
) -> Result<Option<sdkwork_models_contract_service::ModelRankingRefreshJobItem>, DomainError> {
    let row = sqlx::query(LOAD_LATEST_MODEL_RANKING_REFRESH_JOB)
        .bind(subject.tenant_id)
        .bind(subject.organization_id)
        .bind(rank_scope)
        .bind(MODEL_RANKING_REFRESH_JOB_TYPE)
        .fetch_optional(pool)
        .await
        .map_err(sql_error)?;

    Ok(row.as_ref().map(refresh_job_item_from_row))
}

async fn load_latest_refresh_job_for_scope(
    pool: &PgPool,
    tenant_id: i64,
    organization_id: i64,
    rank_scope: &str,
) -> Result<Option<sdkwork_models_contract_service::ModelRankingRefreshJobItem>, DomainError> {
    let row = sqlx::query(LOAD_LATEST_MODEL_RANKING_REFRESH_JOB_FOR_SCOPE)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(rank_scope)
        .bind(MODEL_RANKING_REFRESH_JOB_TYPE)
        .fetch_optional(pool)
        .await
        .map_err(sql_error)?;

    Ok(row.as_ref().map(refresh_job_item_from_row))
}

fn snapshot_metadata(
    row: &sqlx::postgres::PgRow,
) -> crate::sql_model_rankings::RankingSnapshotMetadata {
    metadata_from_json(
        optional_string_cell(row, "metadata").as_deref(),
        string_cell(row, "snapshot_date"),
        parse_period_cell(&string_cell(row, "snapshot_period")),
    )
}

fn ranking_item_from_row(index: usize, row: &sqlx::postgres::PgRow) -> ModelRankingItem {
    let snapshot_date = string_cell(row, "snapshot_date");
    let rank = integer_cell(row, "rank_no");
    let name = string_cell(row, "model");
    ModelRankingItem {
        observed_at: snapshot_date.clone(),
        id: stable_item_id(&string_cell(row, "catalog_key"), &name),
        rank: if rank > 0 { rank } else { index as i64 + 1 },
        prev_rank: integer_cell(row, "previous_rank_no"),
        name,
        vendor: string_cell(row, "vendor"),
        vendor_code: string_cell(row, "vendor_code"),
        modality: modality_label(optional_integer_cell(row, "modality")),
        base_volume: integer_cell(row, "base_volume"),
        cost_indicator: contract_cost_indicator(integer_cell(row, "cost_indicator")),
        latency: integer_cell(row, "latency_p50_ms"),
        context_size: optional_string_cell(row, "context_size_text"),
        is_new: bool_cell(row, "is_new"),
        color: string_cell(row, "color_token"),
        win_rate: optional_decimal_cell(row, "win_rate"),
        pricing: optional_string_cell(row, "pricing_text"),
        license: license_label(optional_integer_cell(row, "license_type")),
        strengths: parse_strengths(optional_string_cell(row, "strengths")),
        requests: integer_cell(row, "request_count"),
        tokens: integer_cell(row, "token_count"),
        cost: decimal_cell(row, "cost_amount"),
        currency: string_cell(row, "currency"),
        trend_score: optional_decimal_cell(row, "trend_score"),
    }
}

fn refresh_job_item_from_row(
    row: &sqlx::postgres::PgRow,
) -> sdkwork_models_contract_service::ModelRankingRefreshJobItem {
    let success_count = integer_cell(row, "success_count");
    let failure_count = integer_cell(row, "failure_count");
    refresh_job_item_from_payload(
        string_cell(row, "id"),
        string_cell(row, "job_name"),
        refresh_job_status_label(
            integer_cell(row, "execution_status"),
            failure_count,
            success_count,
        ),
        integer_cell(row, "tenant_id"),
        integer_cell(row, "organization_id"),
        string_cell(row, "started_at"),
        string_cell(row, "ended_at"),
        integer_cell(row, "duration_ms"),
        success_count,
        failure_count,
        optional_string_cell(row, "failure_reason"),
        optional_string_cell(row, "payload").as_deref(),
    )
}

fn stable_item_id(catalog_key: &str, model: &str) -> String {
    if catalog_key.trim().is_empty() {
        model.to_owned()
    } else {
        catalog_key.to_owned()
    }
}

fn contract_cost_indicator(value: i64) -> i64 {
    if (1..=5).contains(&value) {
        value
    } else {
        3
    }
}

fn string_cell(row: &sqlx::postgres::PgRow, column: &str) -> String {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
}

fn integer_cell(row: &sqlx::postgres::PgRow, column: &str) -> i64 {
    optional_integer_cell(row, column).unwrap_or(0)
}

fn optional_integer_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<i64> {
    row.try_get::<Option<i64>, _>(column)
        .ok()
        .flatten()
        .or_else(|| {
            string_cell(row, column)
                .parse::<f64>()
                .ok()
                .map(|value| value as i64)
        })
}

fn decimal_cell(row: &sqlx::postgres::PgRow, column: &str) -> f64 {
    optional_decimal_cell(row, column).unwrap_or(0.0)
}

fn optional_decimal_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<f64> {
    string_cell(row, column).parse::<f64>().ok()
}

fn bool_cell(row: &sqlx::postgres::PgRow, column: &str) -> bool {
    row.try_get::<Option<bool>, _>(column)
        .ok()
        .flatten()
        .or_else(|| optional_integer_cell(row, column).map(|value| value != 0))
        .unwrap_or(false)
}

fn sql_error(error: sqlx::Error) -> DomainError {
    DomainError::new(error.to_string())
}

fn normalize_subject(subject: ModelRankingsSubject) -> ModelRankingsSubject {
    let (tenant_id, organization_id) =
        normalize_scope_ids(subject.tenant_id, subject.organization_id);
    ModelRankingsSubject {
        tenant_id,
        organization_id,
        user_id: subject.user_id.max(0),
    }
}
