use sha2::{Digest, Sha256};
use sqlx::{Postgres, Sqlite, Transaction};

use sdkwork_models_contract_service::{DomainError, DomainResult};

use crate::runtime_id::next_claw_runtime_id;

pub(crate) const AI_ROUTING_CONFIG_SCOPE: &str = "routing";
const GLOBAL_ROUTING_CONFIG_TENANT_ID: i64 = 0;
const GLOBAL_ROUTING_CONFIG_ORGANIZATION_ID: i64 = 0;

pub(crate) struct AiRoutingConfigChange<'a> {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub operator_id: i64,
    pub request_id: &'a str,
    pub requested_at: &'a str,
    pub changed_object_type: &'a str,
    pub changed_object_id: i64,
    pub action: &'a str,
    pub event_payload: serde_json::Value,
}

pub(crate) async fn record_sqlite_ai_routing_config_change(
    tx: &mut Transaction<'_, Sqlite>,
    change: AiRoutingConfigChange<'_>,
) -> DomainResult<i64> {
    let config_version = bump_sqlite_ai_routing_config_version(
        tx,
        &change,
        AI_ROUTING_CONFIG_SCOPE,
        change.tenant_id,
        change.organization_id,
    )
    .await?;
    if change.tenant_id != GLOBAL_ROUTING_CONFIG_TENANT_ID
        || change.organization_id != GLOBAL_ROUTING_CONFIG_ORGANIZATION_ID
    {
        bump_sqlite_ai_routing_config_version(
            tx,
            &change,
            AI_ROUTING_CONFIG_SCOPE,
            GLOBAL_ROUTING_CONFIG_TENANT_ID,
            GLOBAL_ROUTING_CONFIG_ORGANIZATION_ID,
        )
        .await?;
    }
    insert_sqlite_ai_routing_config_change_event(
        tx,
        &change,
        AI_ROUTING_CONFIG_SCOPE,
        config_version,
    )
    .await?;
    Ok(config_version)
}

pub(crate) async fn record_postgres_ai_routing_config_change(
    tx: &mut Transaction<'_, Postgres>,
    change: AiRoutingConfigChange<'_>,
) -> DomainResult<i64> {
    let config_version = bump_postgres_ai_routing_config_version(
        tx,
        &change,
        AI_ROUTING_CONFIG_SCOPE,
        change.tenant_id,
        change.organization_id,
    )
    .await?;
    if change.tenant_id != GLOBAL_ROUTING_CONFIG_TENANT_ID
        || change.organization_id != GLOBAL_ROUTING_CONFIG_ORGANIZATION_ID
    {
        bump_postgres_ai_routing_config_version(
            tx,
            &change,
            AI_ROUTING_CONFIG_SCOPE,
            GLOBAL_ROUTING_CONFIG_TENANT_ID,
            GLOBAL_ROUTING_CONFIG_ORGANIZATION_ID,
        )
        .await?;
    }
    insert_postgres_ai_routing_config_change_event(
        tx,
        &change,
        AI_ROUTING_CONFIG_SCOPE,
        config_version,
    )
    .await?;
    Ok(config_version)
}

async fn bump_sqlite_ai_routing_config_version(
    tx: &mut Transaction<'_, Sqlite>,
    change: &AiRoutingConfigChange<'_>,
    config_scope: &str,
    tenant_id: i64,
    organization_id: i64,
) -> DomainResult<i64> {
    let uuid = config_version_uuid(tenant_id, organization_id, config_scope);
    let id = next_claw_runtime_id("ai_config_version")?;
    sqlx::query(
        r#"
        INSERT INTO ai_config_version
            (uuid, tenant_id, organization_id, status, config_scope, config_version,
             changed_object_type, changed_object_id, published_at, created_at, updated_at, id)
        VALUES
            (?, ?, ?, 1, ?, 1, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(tenant_id, organization_id, config_scope)
        DO UPDATE SET
            config_version = ai_config_version.config_version + 1,
            changed_object_type = excluded.changed_object_type,
            changed_object_id = excluded.changed_object_id,
            published_at = excluded.published_at,
            updated_at = excluded.updated_at,
            status = 1,
            version = ai_config_version.version + 1
        "#,
    )
    .bind(uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(config_scope)
    .bind(change.changed_object_type)
    .bind(change.changed_object_id)
    .bind(change.requested_at)
    .bind(change.requested_at)
    .bind(change.requested_at)
    .bind(id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to bump AI routing config version", error))?;

    sqlx::query_scalar(
        r#"
        SELECT config_version
        FROM ai_config_version
        WHERE tenant_id = ?
          AND organization_id = ?
          AND config_scope = ?
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(config_scope)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load AI routing config version", error))
}

async fn bump_postgres_ai_routing_config_version(
    tx: &mut Transaction<'_, Postgres>,
    change: &AiRoutingConfigChange<'_>,
    config_scope: &str,
    tenant_id: i64,
    organization_id: i64,
) -> DomainResult<i64> {
    let uuid = config_version_uuid(tenant_id, organization_id, config_scope);
    let id = next_claw_runtime_id("ai_config_version")?;
    sqlx::query_scalar(
        r#"
        INSERT INTO ai_config_version
            (uuid, tenant_id, organization_id, status, config_scope, config_version,
             changed_object_type, changed_object_id, published_at, created_at, updated_at, id)
        VALUES
            ($1, $2, $3, 1, $4, 1, $5, $6, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10)
        ON CONFLICT(tenant_id, organization_id, config_scope)
        DO UPDATE SET
            config_version = ai_config_version.config_version + 1,
            changed_object_type = excluded.changed_object_type,
            changed_object_id = excluded.changed_object_id,
            published_at = excluded.published_at,
            updated_at = excluded.updated_at,
            status = 1,
            version = ai_config_version.version + 1
        RETURNING config_version
        "#,
    )
    .bind(uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(config_scope)
    .bind(change.changed_object_type)
    .bind(change.changed_object_id)
    .bind(change.requested_at)
    .bind(change.requested_at)
    .bind(change.requested_at)
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| store_error("failed to bump AI routing config version", error))
}

async fn insert_sqlite_ai_routing_config_change_event(
    tx: &mut Transaction<'_, Sqlite>,
    change: &AiRoutingConfigChange<'_>,
    config_scope: &str,
    config_version: i64,
) -> DomainResult<()> {
    let event_payload = event_payload(change)?;
    let event_payload_json = event_payload.to_string();
    let payload_hash = digest_hex(&event_payload_json);
    let uuid = config_change_event_uuid(
        change.tenant_id,
        change.organization_id,
        config_scope,
        config_version,
        change.changed_object_type,
        change.changed_object_id,
        change.request_id,
    );
    let id = next_claw_runtime_id("ai_config_change_event")?;
    sqlx::query(
        r#"
        INSERT INTO ai_config_change_event
            (uuid, tenant_id, organization_id, user_id, request_id, payload_hash, status,
             config_scope, changed_object_type, changed_object_id, config_version,
             event_status, event_payload, published_at, created_at, id)
        VALUES
            (?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, 'pending', ?, ?, ?, ?)
        "#,
    )
    .bind(uuid)
    .bind(change.tenant_id)
    .bind(change.organization_id)
    .bind(change.operator_id)
    .bind(change.request_id)
    .bind(payload_hash)
    .bind(config_scope)
    .bind(change.changed_object_type)
    .bind(change.changed_object_id)
    .bind(config_version)
    .bind(event_payload_json)
    .bind(change.requested_at)
    .bind(change.requested_at)
    .bind(id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to write AI routing config change event", error))?;
    Ok(())
}

async fn insert_postgres_ai_routing_config_change_event(
    tx: &mut Transaction<'_, Postgres>,
    change: &AiRoutingConfigChange<'_>,
    config_scope: &str,
    config_version: i64,
) -> DomainResult<()> {
    let event_payload = event_payload(change)?;
    let event_payload_json = event_payload.to_string();
    let payload_hash = digest_hex(&event_payload_json);
    let uuid = config_change_event_uuid(
        change.tenant_id,
        change.organization_id,
        config_scope,
        config_version,
        change.changed_object_type,
        change.changed_object_id,
        change.request_id,
    );
    let id = next_claw_runtime_id("ai_config_change_event")?;
    sqlx::query(
        r#"
        INSERT INTO ai_config_change_event
            (uuid, tenant_id, organization_id, user_id, request_id, payload_hash, status,
             config_scope, changed_object_type, changed_object_id, config_version,
             event_status, event_payload, published_at, created_at, id)
        VALUES
            ($1, $2, $3, $4, $5, $6, 1, $7, $8, $9, $10, 'pending',
             $11::jsonb, $12::timestamptz, $13::timestamptz, $14)
        "#,
    )
    .bind(uuid)
    .bind(change.tenant_id)
    .bind(change.organization_id)
    .bind(change.operator_id)
    .bind(change.request_id)
    .bind(payload_hash)
    .bind(config_scope)
    .bind(change.changed_object_type)
    .bind(change.changed_object_id)
    .bind(config_version)
    .bind(event_payload_json)
    .bind(change.requested_at)
    .bind(change.requested_at)
    .bind(id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to write AI routing config change event", error))?;
    Ok(())
}

fn event_payload(change: &AiRoutingConfigChange<'_>) -> DomainResult<serde_json::Value> {
    let mut payload = match change.event_payload.clone() {
        serde_json::Value::Object(payload) => payload,
        _ => {
            return Err(DomainError::new(
                "AI routing config change event payload must be an object",
            ));
        }
    };
    payload.insert(
        "action".to_owned(),
        serde_json::Value::String(change.action.to_owned()),
    );
    payload.insert(
        "changedObjectType".to_owned(),
        serde_json::Value::String(change.changed_object_type.to_owned()),
    );
    payload.insert(
        "changedObjectId".to_owned(),
        serde_json::Value::Number(change.changed_object_id.into()),
    );
    Ok(serde_json::Value::Object(payload))
}

fn config_version_uuid(tenant_id: i64, organization_id: i64, config_scope: &str) -> String {
    digest_id(
        "routing-config-version",
        format!("{tenant_id}:{organization_id}:{config_scope}").as_str(),
    )
}

fn config_change_event_uuid(
    tenant_id: i64,
    organization_id: i64,
    config_scope: &str,
    config_version: i64,
    changed_object_type: &str,
    changed_object_id: i64,
    request_id: &str,
) -> String {
    digest_id(
        "routing-config-event",
        format!(
            "{tenant_id}:{organization_id}:{config_scope}:{config_version}:{changed_object_type}:{changed_object_id}:{request_id}"
        )
        .as_str(),
    )
}

fn digest_id(prefix: &str, payload: &str) -> String {
    let digest = digest_hex(payload);
    format!("{prefix}-{}", &digest[..24])
}

fn digest_hex(payload: &str) -> String {
    let digest = Sha256::digest(payload.as_bytes());
    hex::encode(digest)
}

fn store_error(context: &str, error: sqlx::Error) -> DomainError {
    DomainError::new(format!("{context}: {error}"))
}
