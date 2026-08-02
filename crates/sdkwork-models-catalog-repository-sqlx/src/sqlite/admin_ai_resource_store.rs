use std::collections::{HashMap, HashSet};

use sdkwork_models_contract_service::{
    AdminAiResourceGroupItem, AdminAiResourceGroupListPage, AdminAiResourceGroupResourceItem,
    AdminAiResourceGroupResourcesPage, AdminAiResourceHierarchyNodeCommand, AdminAiResourceItem,
    AdminAiResourceListPage, AdminAiResourceMemberCommand, AdminAiResourceMemberItem,
    AdminAiResourceReadFuture, AdminAiResourceStore, CreateAdminAiResourceCommand,
    CreateAdminAiResourceGroupCommand, DeleteAdminAiResourceGroupCommand,
    DeleteAdminAiResourceGroupMemberCommand, DomainError, DomainResult,
    ListAdminAiResourceGroupResourcesQuery, ListAdminAiResourceGroupsQuery,
    ListAdminAiResourcesQuery, ReplaceAdminAiResourceHierarchyCommand,
    UpdateAdminAiResourceCommand, UpdateAdminAiResourceGroupCommand,
    UpsertAdminAiResourceGroupMemberCommand,
};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool, Transaction};

use crate::admin_ai_resource_hierarchy::{
    hierarchy_node_schema, resource_code_is_owned, validate_hierarchy_command,
};
use crate::routing_config_change::{record_sqlite_ai_routing_config_change, AiRoutingConfigChange};
use crate::runtime_id::next_claw_runtime_id;

const AI_RESOURCE_TARGET_TYPE: i32 = 91;
const MAX_RESOURCE_GROUP_MEMBERS: i64 = 512;

/// Client-local SQLite schema for the AI resource hierarchy. The models
/// module database contract is postgres-only; the desktop standalone runtime
/// persists channels through this store, so the store owns its schema and
/// creates it on startup instead of a lifecycle migration.
const AI_RESOURCE_STORE_SQLITE_SCHEMA: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS ai_resource (
        id INTEGER PRIMARY KEY,
        uuid TEXT NOT NULL,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        data_scope INTEGER NOT NULL,
        status INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        version INTEGER NOT NULL DEFAULT 0,
        metadata TEXT NOT NULL DEFAULT '{}',
        deleted_at TEXT,
        resource_code TEXT NOT NULL,
        resource_type TEXT NOT NULL,
        display_name TEXT,
        vendor_code TEXT,
        modality_code TEXT,
        api_code TEXT,
        catalog_key TEXT,
        model TEXT,
        provider_native_model TEXT,
        resource_schema TEXT,
        description TEXT,
        sort_order INTEGER
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS ai_resource_group (
        id INTEGER PRIMARY KEY,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        group_code TEXT NOT NULL,
        selection_mode TEXT,
        deleted_at TEXT
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS ai_resource_group_item (
        id INTEGER PRIMARY KEY,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        status INTEGER NOT NULL,
        deleted_at TEXT,
        resource_group_code TEXT NOT NULL,
        resource_code TEXT NOT NULL DEFAULT '',
        child_resource_group_code TEXT NOT NULL DEFAULT '',
        item_role TEXT,
        metadata TEXT,
        sort_order INTEGER
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS ops_audit_log (
        id INTEGER PRIMARY KEY,
        uuid TEXT NOT NULL UNIQUE,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        action TEXT NOT NULL,
        target_type INTEGER NOT NULL,
        target_id INTEGER NOT NULL,
        request_id TEXT NOT NULL,
        operator_id INTEGER NOT NULL,
        operator_type INTEGER NOT NULL,
        change_summary TEXT NOT NULL
    )
    "#,
];

struct ResolvedAiResourceGroupMember {
    item_type: &'static str,
    resource_id: Option<i64>,
    resource_code: String,
    child_resource_group_id: Option<i64>,
    child_resource_group_code: String,
}

#[derive(Debug, Clone)]
pub struct SqliteAdminAiResourceStore {
    pool: SqlitePool,
}

impl SqliteAdminAiResourceStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates the client-local tables when they do not exist. Idempotent and
    /// safe to call on every gateway startup; the desktop assembly invokes it
    /// before serving so channel persistence never hits a missing table.
    pub async fn initialize_schema(&self) -> Result<(), String> {
        for statement in AI_RESOURCE_STORE_SQLITE_SCHEMA {
            // Static DDL constants only; AssertSqlSafe documents the audit.
            sqlx::query(sqlx::AssertSqlSafe(*statement))
                .execute(&self.pool)
                .await
                .map_err(|error| format!("failed to initialize AI resource schema: {error}"))?;
        }
        Ok(())
    }
}

impl AdminAiResourceStore for SqliteAdminAiResourceStore {
    fn list_ai_resources<'a>(
        &'a self,
        query: ListAdminAiResourcesQuery,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceListPage> {
        Box::pin(async move { list_ai_resources(&self.pool, query).await })
    }

    fn create_ai_resource<'a>(
        &'a self,
        command: CreateAdminAiResourceCommand,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceItem> {
        Box::pin(async move {
            let mut tx =
                self.pool.begin().await.map_err(|error| {
                    store_error("failed to begin AI resource transaction", error)
                })?;
            let resource_id = insert_ai_resource(&mut tx, &command).await?;
            replace_members_for_create(&mut tx, resource_id, &command).await?;
            insert_audit_log(
                &mut tx,
                &command.audit_log_uuid,
                &command.request_id,
                command.subject.tenant_id,
                command.subject.organization_id,
                command.subject.operator_id,
                command.subject.operator_type,
                "create_ai_resource",
                resource_id,
                serde_json::json!({
                    "action": "create_ai_resource",
                    "resourceId": resource_id,
                    "resourceCode": &command.resource_code,
                    "resourceType": &command.resource_type,
                    "status": &command.status,
                    "memberCount": command.members.len()
                }),
            )
            .await?;
            record_sqlite_ai_routing_config_change(
                &mut tx,
                ai_resource_routing_config_change(
                    command.subject.tenant_id,
                    command.subject.organization_id,
                    command.subject.operator_id,
                    &command.request_id,
                    &command.requested_at,
                    "create_ai_resource",
                    resource_id,
                    serde_json::json!({
                        "resourceId": resource_id,
                        "resourceCode": &command.resource_code,
                        "resourceType": &command.resource_type,
                        "status": &command.status,
                        "memberCount": command.members.len()
                    }),
                ),
            )
            .await?;
            let item = load_resource_by_id(
                &mut tx,
                resource_id,
                command.subject.tenant_id,
                command.subject.organization_id,
            )
            .await?
            .ok_or_else(|| DomainError::new("created AI resource could not be reloaded"))?;
            tx.commit()
                .await
                .map_err(|error| store_error("failed to commit AI resource transaction", error))?;
            Ok(item)
        })
    }

    fn update_ai_resource<'a>(
        &'a self,
        command: UpdateAdminAiResourceCommand,
    ) -> AdminAiResourceReadFuture<'a, Option<AdminAiResourceItem>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(|error| {
                store_error("failed to begin AI resource update transaction", error)
            })?;
            let Some(current) = load_resource_by_id(
                &mut tx,
                command.resource_id,
                command.subject.tenant_id,
                command.subject.organization_id,
            )
            .await?
            else {
                return Ok(None);
            };
            update_ai_resource_core(&mut tx, &command).await?;
            let effective_resource_code = command
                .resource_code
                .as_deref()
                .unwrap_or(current.resource_code.as_str());
            if let Some(members) = command.members.as_ref() {
                replace_members_for_update(
                    &mut tx,
                    &current.resource_code,
                    effective_resource_code,
                    members,
                    &command,
                )
                .await?;
            } else if command.resource_code.is_some()
                && effective_resource_code != current.resource_code.as_str()
            {
                rename_members_for_resource_code(
                    &mut tx,
                    command.resource_id,
                    &current.resource_code,
                    effective_resource_code,
                    &command,
                )
                .await?;
            }
            if command.status.is_some() {
                sync_resource_group_status(
                    &mut tx,
                    effective_resource_code,
                    &command,
                    status_code(command.status.as_deref().unwrap_or("active")),
                )
                .await?;
            }
            insert_audit_log(
                &mut tx,
                &command.audit_log_uuid,
                &command.request_id,
                command.subject.tenant_id,
                command.subject.organization_id,
                command.subject.operator_id,
                command.subject.operator_type,
                "update_ai_resource",
                command.resource_id,
                serde_json::json!({
                    "action": "update_ai_resource",
                    "resourceId": command.resource_id,
                    "resourceCodeChanged": command.resource_code.is_some(),
                    "statusChanged": command.status.is_some(),
                    "membersChanged": command.members.is_some()
                }),
            )
            .await?;
            record_sqlite_ai_routing_config_change(
                &mut tx,
                ai_resource_routing_config_change(
                    command.subject.tenant_id,
                    command.subject.organization_id,
                    command.subject.operator_id,
                    &command.request_id,
                    &command.requested_at,
                    "update_ai_resource",
                    command.resource_id,
                    serde_json::json!({
                        "resourceId": command.resource_id,
                        "resourceCodeChanged": command.resource_code.is_some(),
                        "resourceTypeChanged": command.resource_type.is_some(),
                        "vendorChanged": command.vendor_code.is_some(),
                        "modalityChanged": command.modality_code.is_some(),
                        "apiChanged": command.api_endpoint_code.is_some(),
                        "modelChanged": command.model.is_some()
                            || command.catalog_key.is_some()
                            || command.provider_native_model.is_some(),
                        "statusChanged": command.status.is_some(),
                        "membersChanged": command.members.is_some()
                    }),
                ),
            )
            .await?;
            let item = load_resource_by_id(
                &mut tx,
                command.resource_id,
                command.subject.tenant_id,
                command.subject.organization_id,
            )
            .await?
            .ok_or_else(|| DomainError::new("updated AI resource could not be reloaded"))?;
            tx.commit().await.map_err(|error| {
                store_error("failed to commit AI resource update transaction", error)
            })?;
            Ok(Some(item))
        })
    }

    fn replace_ai_resource_hierarchy<'a>(
        &'a self,
        command: ReplaceAdminAiResourceHierarchyCommand,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceItem> {
        Box::pin(async move { replace_ai_resource_hierarchy(&self.pool, command).await })
    }

    fn list_ai_resource_groups<'a>(
        &'a self,
        query: ListAdminAiResourceGroupsQuery,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceGroupListPage> {
        Box::pin(async move { list_ai_resource_groups(&self.pool, query).await })
    }

    fn list_ai_resource_group_resources<'a>(
        &'a self,
        query: ListAdminAiResourceGroupResourcesQuery,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceGroupResourcesPage> {
        Box::pin(async move { list_ai_resource_group_resources(&self.pool, query).await })
    }

    fn create_ai_resource_group<'a>(
        &'a self,
        command: CreateAdminAiResourceGroupCommand,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceGroupItem> {
        Box::pin(async move { create_ai_resource_group(&self.pool, command).await })
    }

    fn update_ai_resource_group<'a>(
        &'a self,
        command: UpdateAdminAiResourceGroupCommand,
    ) -> AdminAiResourceReadFuture<'a, Option<AdminAiResourceGroupItem>> {
        Box::pin(async move { update_ai_resource_group(&self.pool, command).await })
    }

    fn upsert_ai_resource_group_member<'a>(
        &'a self,
        command: UpsertAdminAiResourceGroupMemberCommand,
    ) -> AdminAiResourceReadFuture<'a, Option<AdminAiResourceGroupResourceItem>> {
        Box::pin(async move { upsert_ai_resource_group_member(&self.pool, command).await })
    }

    fn delete_ai_resource_group_member<'a>(
        &'a self,
        command: DeleteAdminAiResourceGroupMemberCommand,
    ) -> AdminAiResourceReadFuture<'a, bool> {
        Box::pin(async move { delete_ai_resource_group_member(&self.pool, command).await })
    }

    fn delete_ai_resource_group<'a>(
        &'a self,
        command: DeleteAdminAiResourceGroupCommand,
    ) -> AdminAiResourceReadFuture<'a, bool> {
        Box::pin(async move { delete_ai_resource_group(&self.pool, command).await })
    }
}

async fn replace_ai_resource_hierarchy(
    pool: &SqlitePool,
    command: ReplaceAdminAiResourceHierarchyCommand,
) -> DomainResult<AdminAiResourceItem> {
    let desired_resource_codes = validate_hierarchy_command(&command)?;
    let mut tx = pool.begin().await.map_err(|error| {
        store_error(
            "failed to begin AI resource hierarchy replacement transaction",
            error,
        )
    })?;
    let mut root_resource_id = None;
    for node in &command.nodes {
        let resource_id = upsert_hierarchy_resource(&mut tx, &command, node).await?;
        replace_hierarchy_node_members(&mut tx, &command, node, resource_id).await?;
        if node.resource_code == command.root_resource_code {
            root_resource_id = Some(resource_id);
        }
    }
    let root_resource_id = root_resource_id
        .ok_or_else(|| DomainError::new("AI resource hierarchy root could not be persisted"))?;
    let retired_count =
        retire_stale_hierarchy_resources(&mut tx, &command, &desired_resource_codes).await?;
    insert_audit_log(
        &mut tx,
        &command.audit_log_uuid,
        &command.request_id,
        command.subject.tenant_id,
        command.subject.organization_id,
        command.subject.operator_id,
        command.subject.operator_type,
        "replace_ai_resource_hierarchy",
        root_resource_id,
        serde_json::json!({
            "action": "replace_ai_resource_hierarchy",
            "rootResourceCode": &command.root_resource_code,
            "resourceCount": command.nodes.len(),
            "retiredResourceCount": retired_count
        }),
    )
    .await?;
    record_sqlite_ai_routing_config_change(
        &mut tx,
        ai_resource_routing_config_change(
            command.subject.tenant_id,
            command.subject.organization_id,
            command.subject.operator_id,
            &command.request_id,
            &command.requested_at,
            "replace_ai_resource_hierarchy",
            root_resource_id,
            serde_json::json!({
                "rootResourceCode": &command.root_resource_code,
                "resourceCount": command.nodes.len(),
                "retiredResourceCount": retired_count
            }),
        ),
    )
    .await?;
    let item = load_resource_by_id(
        &mut tx,
        root_resource_id,
        command.subject.tenant_id,
        command.subject.organization_id,
    )
    .await?
    .ok_or_else(|| DomainError::new("replaced AI resource hierarchy root could not be reloaded"))?;
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource hierarchy replacement transaction",
            error,
        )
    })?;
    Ok(item)
}

async fn list_ai_resources(
    pool: &SqlitePool,
    query: ListAdminAiResourcesQuery,
) -> DomainResult<AdminAiResourceListPage> {
    let search = resource_search_pattern(query.q.as_deref());
    let status = query.status.as_deref().map(status_code);
    let total_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM ai_resource
        WHERE (
                (tenant_id = ?1 AND organization_id = ?2)
                OR (tenant_id = 0 AND organization_id = 0)
              )
          AND deleted_at IS NULL
          AND NOT (
              tenant_id = 0
              AND organization_id = 0
              AND (?1 <> 0 OR ?2 <> 0)
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource tenant_resource
                  WHERE tenant_resource.tenant_id = ?1
                    AND tenant_resource.organization_id = ?2
                    AND tenant_resource.resource_code = ai_resource.resource_code
                    AND tenant_resource.deleted_at IS NULL
              )
          )
          AND (
              ?3 IS NULL
              OR resource_code LIKE ?3
              OR COALESCE(NULLIF(display_name, ''), resource_code) LIKE ?3
              OR COALESCE(resource_type, '') LIKE ?3
              OR COALESCE(vendor_code, '') LIKE ?3
              OR COALESCE(modality_code, '') LIKE ?3
              OR COALESCE(api_code, '') LIKE ?3
              OR COALESCE(catalog_key, '') LIKE ?3
              OR COALESCE(model, '') LIKE ?3
              OR COALESCE(provider_native_model, '') LIKE ?3
              OR COALESCE(description, '') LIKE ?3
              OR COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '') LIKE ?3
              OR COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.description'), '') LIKE ?3
              OR EXISTS (
                  SELECT 1
                  FROM ai_resource_group_item member_item
                  JOIN ai_resource member_resource
                    ON member_resource.tenant_id = member_item.tenant_id
                   AND member_resource.organization_id = member_item.organization_id
                   AND member_resource.resource_code = member_item.resource_code
                   AND member_resource.deleted_at IS NULL
                  WHERE member_item.tenant_id = ai_resource.tenant_id
                    AND member_item.organization_id = ai_resource.organization_id
                    AND member_item.resource_group_code = ai_resource.resource_code
                    AND member_item.deleted_at IS NULL
                    AND member_item.status = 1
                    AND (
                        COALESCE(member_resource.vendor_code, '') LIKE ?3
                        OR COALESCE(member_resource.catalog_key, '') LIKE ?3
                        OR COALESCE(member_resource.model, '') LIKE ?3
                        OR COALESCE(member_resource.display_name, '') LIKE ?3
                    )
              )
          )
          AND (?4 IS NULL OR resource_type = ?4)
          AND (?5 IS NULL OR status = ?5)
          AND (
              ?6 IS NULL
              OR lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.accessChannelKind'), '')) = ?6
          )
          AND (
              ?7 IS NULL
              OR lower(COALESCE(vendor_code, '')) = ?7
              OR EXISTS (
                  SELECT 1
                  FROM ai_resource_group_item vendor_item
                  JOIN ai_resource vendor_resource
                    ON vendor_resource.tenant_id = vendor_item.tenant_id
                   AND vendor_resource.organization_id = vendor_item.organization_id
                   AND vendor_resource.resource_code = vendor_item.resource_code
                   AND vendor_resource.deleted_at IS NULL
                  WHERE vendor_item.tenant_id = ai_resource.tenant_id
                    AND vendor_item.organization_id = ai_resource.organization_id
                    AND vendor_item.resource_group_code = ai_resource.resource_code
                    AND vendor_item.deleted_at IS NULL
                    AND vendor_item.status = 1
                    AND lower(COALESCE(vendor_resource.vendor_code, '')) = ?7
              )
          )
          AND (
              ?8 IS NULL
              OR COALESCE(json_array_length(json_extract(COALESCE(resource_schema, '{}'), '$.supportedAgentProviderIds')), 0) = 0
              OR EXISTS (
                  SELECT 1
                  FROM json_each(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.supportedAgentProviderIds'), '[]')) provider
                  WHERE lower(CAST(provider.value AS TEXT)) = ?8
              )
          )
          AND (
              ?9 = 0
              OR (
                  lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.accessChannelKind'), '')) IN ('official', 'relay')
                  AND (
                      lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '')) LIKE 'http://%'
                      OR lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '')) LIKE 'https://%'
                  )
              )
          )
        "#,
    )
    .bind(query.subject.tenant_id)
    .bind(query.subject.organization_id)
    .bind(search.as_deref())
    .bind(query.resource_type.as_deref())
    .bind(status)
    .bind(query.access_channel_kind.as_deref())
    .bind(query.vendor_code.as_deref())
    .bind(query.agent_provider_id.as_deref())
    .bind(query.require_valid_access_channel_metadata)
    .fetch_one(pool)
    .await
    .map_err(|error| store_error("failed to count AI resources", error))?;
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            resource_code,
            resource_type AS resource_type,
            COALESCE(NULLIF(display_name, ''), resource_code) AS display_name,
            vendor_code,
            modality_code,
            api_code AS api_endpoint_code,
            catalog_key,
            model,
            provider_native_model,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.accessChannelKind'), '') AS access_channel_kind,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '') AS base_url,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.defaultVendorCode'), '') AS default_vendor_code,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.defaultModelId'), '') AS default_model_id,
            COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.supportedAgentProviderIds'), '[]') AS supported_agent_provider_ids_json,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.contextTokens') IN ('integer', 'real')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.contextTokens') AS INTEGER)
            END AS context_tokens,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.maxOutputTokens') IN ('integer', 'real')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.maxOutputTokens') AS INTEGER)
            END AS max_output_tokens,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.toolCallRounds') IN ('integer', 'real')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.toolCallRounds') AS INTEGER)
            END AS tool_call_rounds,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.supportsMultimodal') IN ('true', 'false')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.supportsMultimodal') AS INTEGER)
            END AS supports_multimodal,
            COALESCE(
                NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.description'), ''),
                NULLIF(description, '')
            ) AS description,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.capability'), '') AS capability,
            COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.capabilities'), '[]') AS capabilities_json,
            COALESCE(
                NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.compositionMode'), ''),
                (
                    SELECT NULLIF(g.selection_mode, '')
                    FROM ai_resource_group g
                    WHERE g.tenant_id = ai_resource.tenant_id
                      AND g.organization_id = ai_resource.organization_id
                      AND g.group_code = ai_resource.resource_code
                      AND g.deleted_at IS NULL
                    LIMIT 1
                ),
                'single'
            ) AS composition_mode,
            status,
            sort_order
        FROM ai_resource
        WHERE (
                (tenant_id = ?1 AND organization_id = ?2)
                OR (tenant_id = 0 AND organization_id = 0)
              )
          AND deleted_at IS NULL
          AND NOT (
              tenant_id = 0
              AND organization_id = 0
              AND (?1 <> 0 OR ?2 <> 0)
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource tenant_resource
                  WHERE tenant_resource.tenant_id = ?1
                    AND tenant_resource.organization_id = ?2
                    AND tenant_resource.resource_code = ai_resource.resource_code
                    AND tenant_resource.deleted_at IS NULL
              )
          )
          AND (
              ?3 IS NULL
              OR resource_code LIKE ?3
              OR COALESCE(NULLIF(display_name, ''), resource_code) LIKE ?3
              OR COALESCE(resource_type, '') LIKE ?3
              OR COALESCE(vendor_code, '') LIKE ?3
              OR COALESCE(modality_code, '') LIKE ?3
              OR COALESCE(api_code, '') LIKE ?3
              OR COALESCE(catalog_key, '') LIKE ?3
              OR COALESCE(model, '') LIKE ?3
              OR COALESCE(provider_native_model, '') LIKE ?3
              OR COALESCE(description, '') LIKE ?3
              OR COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '') LIKE ?3
              OR COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.description'), '') LIKE ?3
              OR EXISTS (
                  SELECT 1
                  FROM ai_resource_group_item member_item
                  JOIN ai_resource member_resource
                    ON member_resource.tenant_id = member_item.tenant_id
                   AND member_resource.organization_id = member_item.organization_id
                   AND member_resource.resource_code = member_item.resource_code
                   AND member_resource.deleted_at IS NULL
                  WHERE member_item.tenant_id = ai_resource.tenant_id
                    AND member_item.organization_id = ai_resource.organization_id
                    AND member_item.resource_group_code = ai_resource.resource_code
                    AND member_item.deleted_at IS NULL
                    AND member_item.status = 1
                    AND (
                        COALESCE(member_resource.vendor_code, '') LIKE ?3
                        OR COALESCE(member_resource.catalog_key, '') LIKE ?3
                        OR COALESCE(member_resource.model, '') LIKE ?3
                        OR COALESCE(member_resource.display_name, '') LIKE ?3
                    )
              )
          )
          AND (?4 IS NULL OR resource_type = ?4)
          AND (?5 IS NULL OR status = ?5)
          AND (
              ?6 IS NULL
              OR lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.accessChannelKind'), '')) = ?6
          )
          AND (
              ?7 IS NULL
              OR lower(COALESCE(vendor_code, '')) = ?7
              OR EXISTS (
                  SELECT 1
                  FROM ai_resource_group_item vendor_item
                  JOIN ai_resource vendor_resource
                    ON vendor_resource.tenant_id = vendor_item.tenant_id
                   AND vendor_resource.organization_id = vendor_item.organization_id
                   AND vendor_resource.resource_code = vendor_item.resource_code
                   AND vendor_resource.deleted_at IS NULL
                  WHERE vendor_item.tenant_id = ai_resource.tenant_id
                    AND vendor_item.organization_id = ai_resource.organization_id
                    AND vendor_item.resource_group_code = ai_resource.resource_code
                    AND vendor_item.deleted_at IS NULL
                    AND vendor_item.status = 1
                    AND lower(COALESCE(vendor_resource.vendor_code, '')) = ?7
              )
          )
          AND (
              ?8 IS NULL
              OR COALESCE(json_array_length(json_extract(COALESCE(resource_schema, '{}'), '$.supportedAgentProviderIds')), 0) = 0
              OR EXISTS (
                  SELECT 1
                  FROM json_each(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.supportedAgentProviderIds'), '[]')) provider
                  WHERE lower(CAST(provider.value AS TEXT)) = ?8
              )
          )
          AND (
              ?9 = 0
              OR (
                  lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.accessChannelKind'), '')) IN ('official', 'relay')
                  AND (
                      lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '')) LIKE 'http://%'
                      OR lower(COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '')) LIKE 'https://%'
                  )
              )
          )
        ORDER BY COALESCE(sort_order, 100000) ASC, id ASC
        LIMIT ?10 OFFSET ?11
        "#,
    )
    .bind(query.subject.tenant_id)
    .bind(query.subject.organization_id)
    .bind(search.as_deref())
    .bind(query.resource_type.as_deref())
    .bind(status)
    .bind(query.access_channel_kind.as_deref())
    .bind(query.vendor_code.as_deref())
    .bind(query.agent_provider_id.as_deref())
    .bind(query.require_valid_access_channel_metadata)
    .bind(query.normalized_limit())
    .bind(query.normalized_offset())
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to list AI resources", error))?;

    let resource_codes = rows
        .iter()
        .map(|row| row.try_get("resource_code").map_err(row_error))
        .collect::<DomainResult<Vec<String>>>()?;
    let members = load_members(
        pool,
        query.subject.tenant_id,
        query.subject.organization_id,
        &resource_codes,
    )
    .await?;

    let items = rows
        .into_iter()
        .map(|row| item_from_row(row, &members))
        .collect::<DomainResult<Vec<_>>>()?;
    Ok(AdminAiResourceListPage { items, total_count })
}

async fn list_ai_resource_groups(
    pool: &SqlitePool,
    query: ListAdminAiResourceGroupsQuery,
) -> DomainResult<AdminAiResourceGroupListPage> {
    let search = resource_search_pattern(query.q.as_deref());
    let total_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM ai_resource_group g
        WHERE (
                (g.tenant_id = ?1 AND g.organization_id = ?2)
                OR (g.tenant_id = 0 AND g.organization_id = 0)
              )
          AND g.deleted_at IS NULL
          AND COALESCE(NULLIF(g.group_type, ''), 'api_group') = 'api_group'
          AND NOT (
              g.tenant_id = 0
              AND g.organization_id = 0
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource_group tenant_group
                  WHERE tenant_group.tenant_id = ?1
                    AND tenant_group.organization_id = ?2
                    AND tenant_group.group_code = g.group_code
                    AND tenant_group.deleted_at IS NULL
                    AND COALESCE(NULLIF(tenant_group.group_type, ''), 'api_group') = 'api_group'
              )
          )
          AND (
              ?3 IS NULL
              OR LOWER(g.group_code) LIKE LOWER(?3)
              OR LOWER(g.group_name) LIKE LOWER(?3)
              OR LOWER(COALESCE(g.description, '')) LIKE LOWER(?3)
          )
        "#,
    )
    .bind(query.subject.tenant_id)
    .bind(query.subject.organization_id)
    .bind(search.as_deref())
    .fetch_one(pool)
    .await
    .map_err(|error| store_error("failed to count AI resource groups", error))?;

    let rows = sqlx::query(
        r#"
        SELECT
            g.id,
            g.group_code,
            g.group_name,
            COALESCE(NULLIF(g.group_type, ''), 'api_group') AS group_type,
            COALESCE(NULLIF(g.selection_mode, ''), 'manual') AS selection_mode,
            g.description,
            g.sort_order,
            g.status,
            CASE
                WHEN g.selection_mode = 'dynamic_all_api' THEN (
                    SELECT COUNT(1)
                    FROM ai_resource r
                    WHERE (
                            (r.tenant_id = g.tenant_id AND r.organization_id = g.organization_id)
                            OR (r.tenant_id = 0 AND r.organization_id = 0)
                          )
                      AND r.resource_type = 'api_endpoint'
                      AND r.deleted_at IS NULL
                      AND NOT (
                          r.tenant_id = 0
                          AND r.organization_id = 0
                          AND (g.tenant_id <> 0 OR g.organization_id <> 0)
                          AND EXISTS (
                              SELECT 1
                              FROM ai_resource tenant_resource
                              WHERE tenant_resource.tenant_id = g.tenant_id
                                AND tenant_resource.organization_id = g.organization_id
                                AND tenant_resource.resource_code = r.resource_code
                                AND tenant_resource.deleted_at IS NULL
                          )
                      )
                )
                ELSE (
                    SELECT COUNT(1)
                    FROM ai_resource_group_item item
                    JOIN ai_resource r
                      ON r.resource_code = item.resource_code
                     AND r.deleted_at IS NULL
                     AND (
                          (r.tenant_id = item.tenant_id AND r.organization_id = item.organization_id)
                          OR (r.tenant_id = 0 AND r.organization_id = 0)
                     )
                     AND NOT (
                          r.tenant_id = 0
                          AND r.organization_id = 0
                          AND (item.tenant_id <> 0 OR item.organization_id <> 0)
                          AND EXISTS (
                              SELECT 1
                              FROM ai_resource tenant_resource
                              WHERE tenant_resource.tenant_id = item.tenant_id
                                AND tenant_resource.organization_id = item.organization_id
                                AND tenant_resource.resource_code = item.resource_code
                                AND tenant_resource.deleted_at IS NULL
                          )
                     )
                    WHERE item.tenant_id = g.tenant_id
                      AND item.organization_id = g.organization_id
                      AND item.resource_group_id = g.id
                      AND item.item_type = 'resource'
                      AND item.deleted_at IS NULL
                      AND item.status = 1
                )
            END AS resource_count,
            CASE WHEN g.selection_mode = 'dynamic_all_api' THEN 1 ELSE 0 END AS dynamic
        FROM ai_resource_group g
        WHERE (
                (g.tenant_id = ?1 AND g.organization_id = ?2)
                OR (g.tenant_id = 0 AND g.organization_id = 0)
              )
          AND g.deleted_at IS NULL
          AND COALESCE(NULLIF(g.group_type, ''), 'api_group') = 'api_group'
          AND NOT (
              g.tenant_id = 0
              AND g.organization_id = 0
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource_group tenant_group
                  WHERE tenant_group.tenant_id = ?1
                    AND tenant_group.organization_id = ?2
                    AND tenant_group.group_code = g.group_code
                    AND tenant_group.deleted_at IS NULL
                    AND COALESCE(NULLIF(tenant_group.group_type, ''), 'api_group') = 'api_group'
              )
          )
          AND (
              ?3 IS NULL
              OR LOWER(g.group_code) LIKE LOWER(?3)
              OR LOWER(g.group_name) LIKE LOWER(?3)
              OR LOWER(COALESCE(g.description, '')) LIKE LOWER(?3)
          )
        ORDER BY CASE WHEN g.tenant_id = ?1 AND g.organization_id = ?2 THEN 0 ELSE 1 END,
                 COALESCE(g.sort_order, 100000) ASC,
                 g.id ASC
        LIMIT ?4 OFFSET ?5
        "#,
    )
    .bind(query.subject.tenant_id)
    .bind(query.subject.organization_id)
    .bind(search.as_deref())
    .bind(query.normalized_limit())
    .bind(query.normalized_offset())
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to list AI resource groups", error))?;
    let mut groups = rows
        .into_iter()
        .map(group_item_from_row)
        .collect::<DomainResult<Vec<_>>>()?;
    hydrate_group_summaries(
        pool,
        query.subject.tenant_id,
        query.subject.organization_id,
        &mut groups,
    )
    .await?;
    Ok(AdminAiResourceGroupListPage {
        items: groups,
        total_count,
    })
}

async fn list_ai_resource_group_resources(
    pool: &SqlitePool,
    query: ListAdminAiResourceGroupResourcesQuery,
) -> DomainResult<AdminAiResourceGroupResourcesPage> {
    let group = load_group_header(
        pool,
        query.subject.tenant_id,
        query.subject.organization_id,
        &query.group_id_or_code,
    )
    .await?
    .ok_or_else(|| DomainError::not_found("AI resource group was not found"))?;
    let search = resource_search_pattern(query.q.as_deref());
    let limit = query.normalized_limit();
    let offset = query.normalized_offset();
    let (total_count, rows) =
        if is_dynamic_group(group.group_code.as_str(), group.selection_mode.as_str()) {
            let total_count: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(1)
                FROM ai_resource r
                WHERE (
                        (r.tenant_id = ?1 AND r.organization_id = ?2)
                        OR (r.tenant_id = 0 AND r.organization_id = 0)
                      )
                  AND r.resource_type = 'api_endpoint'
                  AND r.deleted_at IS NULL
                  AND NOT (
                      r.tenant_id = 0
                      AND r.organization_id = 0
                      AND (?1 <> 0 OR ?2 <> 0)
                      AND EXISTS (
                          SELECT 1
                          FROM ai_resource tenant_resource
                          WHERE tenant_resource.tenant_id = ?1
                            AND tenant_resource.organization_id = ?2
                            AND tenant_resource.resource_code = r.resource_code
                            AND tenant_resource.deleted_at IS NULL
                      )
                  )
                  AND (
                      ?3 IS NULL
                      OR r.resource_code LIKE ?3
                      OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) LIKE ?3
                      OR COALESCE(r.resource_type, '') LIKE ?3
                      OR COALESCE(r.vendor_code, '') LIKE ?3
                      OR COALESCE(r.modality_code, '') LIKE ?3
                      OR COALESCE(r.api_code, '') LIKE ?3
                      OR COALESCE(r.catalog_key, '') LIKE ?3
                      OR COALESCE(r.model, '') LIKE ?3
                      OR COALESCE(r.provider_native_model, '') LIKE ?3
                  )
                "#,
            )
            .bind(group.tenant_id)
            .bind(group.organization_id)
            .bind(search.as_deref())
            .fetch_one(pool)
            .await
            .map_err(|error| store_error("failed to count AI resource group resources", error))?;
            let rows = sqlx::query(
                r#"
                SELECT
                    r.id,
                    r.resource_code,
                    r.resource_type,
                    COALESCE(NULLIF(r.display_name, ''), r.resource_code) AS display_name,
                    r.vendor_code,
                    r.modality_code,
                    r.api_code AS api_endpoint_code,
                    r.catalog_key,
                    r.model,
                    r.provider_native_model,
                    r.status,
                    r.sort_order,
                    'included' AS member_role
                FROM ai_resource r
                WHERE (
                        (r.tenant_id = ?1 AND r.organization_id = ?2)
                        OR (r.tenant_id = 0 AND r.organization_id = 0)
                      )
                  AND r.resource_type = 'api_endpoint'
                  AND r.deleted_at IS NULL
                  AND NOT (
                      r.tenant_id = 0
                      AND r.organization_id = 0
                      AND (?1 <> 0 OR ?2 <> 0)
                      AND EXISTS (
                          SELECT 1
                          FROM ai_resource tenant_resource
                          WHERE tenant_resource.tenant_id = ?1
                            AND tenant_resource.organization_id = ?2
                            AND tenant_resource.resource_code = r.resource_code
                            AND tenant_resource.deleted_at IS NULL
                      )
                  )
                  AND (
                      ?3 IS NULL
                      OR r.resource_code LIKE ?3
                      OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) LIKE ?3
                      OR COALESCE(r.resource_type, '') LIKE ?3
                      OR COALESCE(r.vendor_code, '') LIKE ?3
                      OR COALESCE(r.modality_code, '') LIKE ?3
                      OR COALESCE(r.api_code, '') LIKE ?3
                      OR COALESCE(r.catalog_key, '') LIKE ?3
                      OR COALESCE(r.model, '') LIKE ?3
                      OR COALESCE(r.provider_native_model, '') LIKE ?3
                  )
                ORDER BY COALESCE(r.sort_order, 100000) ASC, r.id ASC
                LIMIT ?4 OFFSET ?5
                "#,
            )
            .bind(group.tenant_id)
            .bind(group.organization_id)
            .bind(search.as_deref())
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|error| store_error("failed to list AI resource group resources", error))?;
            (total_count, rows)
        } else {
            let total_count: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(1)
                FROM ai_resource_group_item i
                JOIN ai_resource r
                  ON r.resource_code = i.resource_code
                 AND r.deleted_at IS NULL
                 AND (
                      (r.tenant_id = i.tenant_id AND r.organization_id = i.organization_id)
                      OR (r.tenant_id = 0 AND r.organization_id = 0)
                 )
                 AND NOT (
                      r.tenant_id = 0
                      AND r.organization_id = 0
                      AND (i.tenant_id <> 0 OR i.organization_id <> 0)
                      AND EXISTS (
                          SELECT 1
                          FROM ai_resource tenant_resource
                          WHERE tenant_resource.tenant_id = i.tenant_id
                            AND tenant_resource.organization_id = i.organization_id
                            AND tenant_resource.resource_code = i.resource_code
                            AND tenant_resource.deleted_at IS NULL
                      )
                 )
                WHERE i.tenant_id = ?1
                  AND i.organization_id = ?2
                  AND i.resource_group_id = ?3
                  AND i.item_type = 'resource'
                  AND i.deleted_at IS NULL
                  AND i.status = 1
                  AND (
                      ?4 IS NULL
                      OR r.resource_code LIKE ?4
                      OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) LIKE ?4
                      OR COALESCE(r.resource_type, '') LIKE ?4
                      OR COALESCE(r.vendor_code, '') LIKE ?4
                      OR COALESCE(r.modality_code, '') LIKE ?4
                      OR COALESCE(r.api_code, '') LIKE ?4
                      OR COALESCE(r.catalog_key, '') LIKE ?4
                      OR COALESCE(r.model, '') LIKE ?4
                      OR COALESCE(r.provider_native_model, '') LIKE ?4
                  )
                "#,
            )
            .bind(group.tenant_id)
            .bind(group.organization_id)
            .bind(group.id)
            .bind(search.as_deref())
            .fetch_one(pool)
            .await
            .map_err(|error| store_error("failed to count AI resource group resources", error))?;
            let rows = sqlx::query(
                r#"
                SELECT
                    r.id,
                    r.resource_code,
                    r.resource_type,
                    COALESCE(NULLIF(r.display_name, ''), r.resource_code) AS display_name,
                    r.vendor_code,
                    r.modality_code,
                    r.api_code AS api_endpoint_code,
                    r.catalog_key,
                    r.model,
                    r.provider_native_model,
                    r.status,
                    COALESCE(i.sort_order, r.sort_order) AS sort_order,
                    COALESCE(NULLIF(i.item_role, ''), 'included') AS member_role
                FROM ai_resource_group_item i
                JOIN ai_resource r
                  ON r.resource_code = i.resource_code
                 AND r.deleted_at IS NULL
                 AND (
                      (r.tenant_id = i.tenant_id AND r.organization_id = i.organization_id)
                      OR (r.tenant_id = 0 AND r.organization_id = 0)
                 )
                 AND NOT (
                      r.tenant_id = 0
                      AND r.organization_id = 0
                      AND (i.tenant_id <> 0 OR i.organization_id <> 0)
                      AND EXISTS (
                          SELECT 1
                          FROM ai_resource tenant_resource
                          WHERE tenant_resource.tenant_id = i.tenant_id
                            AND tenant_resource.organization_id = i.organization_id
                            AND tenant_resource.resource_code = i.resource_code
                            AND tenant_resource.deleted_at IS NULL
                      )
                 )
                WHERE i.tenant_id = ?1
                  AND i.organization_id = ?2
                  AND i.resource_group_id = ?3
                  AND i.item_type = 'resource'
                  AND i.deleted_at IS NULL
                  AND i.status = 1
                  AND (
                      ?4 IS NULL
                      OR r.resource_code LIKE ?4
                      OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) LIKE ?4
                      OR COALESCE(r.resource_type, '') LIKE ?4
                      OR COALESCE(r.vendor_code, '') LIKE ?4
                      OR COALESCE(r.modality_code, '') LIKE ?4
                      OR COALESCE(r.api_code, '') LIKE ?4
                      OR COALESCE(r.catalog_key, '') LIKE ?4
                      OR COALESCE(r.model, '') LIKE ?4
                      OR COALESCE(r.provider_native_model, '') LIKE ?4
                  )
                ORDER BY COALESCE(i.sort_order, r.sort_order, 100000) ASC, i.id ASC
                LIMIT ?5 OFFSET ?6
                "#,
            )
            .bind(group.tenant_id)
            .bind(group.organization_id)
            .bind(group.id)
            .bind(search.as_deref())
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|error| store_error("failed to list AI resource group resources", error))?;
            (total_count, rows)
        };

    let items = rows
        .into_iter()
        .map(group_resource_from_row)
        .collect::<DomainResult<Vec<_>>>()?;
    Ok(AdminAiResourceGroupResourcesPage { items, total_count })
}

async fn create_ai_resource_group(
    pool: &SqlitePool,
    command: CreateAdminAiResourceGroupCommand,
) -> DomainResult<AdminAiResourceGroupItem> {
    let mut tx = pool.begin().await.map_err(|error| {
        store_error(
            "failed to begin AI resource group create transaction",
            error,
        )
    })?;
    let group_id = insert_ai_resource_group(&mut tx, &command).await?;
    if is_dynamic_group(&command.group_code, &command.selection_mode) && !command.members.is_empty()
    {
        return Err(DomainError::conflict(
            "dynamic API groups cannot maintain resource relationships",
        ));
    }
    replace_group_members_for_create(&mut tx, group_id, &command).await?;
    insert_audit_log(
        &mut tx,
        &command.audit_log_uuid,
        &command.request_id,
        command.subject.tenant_id,
        command.subject.organization_id,
        command.subject.operator_id,
        command.subject.operator_type,
        "create_ai_resource_group",
        group_id,
        serde_json::json!({
            "action": "create_ai_resource_group",
            "groupId": group_id,
            "groupCode": &command.group_code,
            "memberCount": command.members.len()
        }),
    )
    .await?;
    record_sqlite_ai_routing_config_change(
        &mut tx,
        ai_resource_group_routing_config_change(
            command.subject.tenant_id,
            command.subject.organization_id,
            command.subject.operator_id,
            &command.request_id,
            &command.requested_at,
            "create_ai_resource_group",
            group_id,
            serde_json::json!({
                "groupId": group_id,
                "groupCode": &command.group_code,
                "memberCount": command.members.len()
            }),
        ),
    )
    .await?;
    let item = load_group_by_id(
        &mut tx,
        group_id,
        command.subject.tenant_id,
        command.subject.organization_id,
    )
    .await?
    .ok_or_else(|| DomainError::new("created AI resource group could not be reloaded"))?;
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource group create transaction",
            error,
        )
    })?;
    Ok(item)
}

async fn update_ai_resource_group(
    pool: &SqlitePool,
    command: UpdateAdminAiResourceGroupCommand,
) -> DomainResult<Option<AdminAiResourceGroupItem>> {
    let mut tx = pool.begin().await.map_err(|error| {
        store_error(
            "failed to begin AI resource group update transaction",
            error,
        )
    })?;
    let Some(current) = load_group_by_id(
        &mut tx,
        command.group_id,
        command.subject.tenant_id,
        command.subject.organization_id,
    )
    .await?
    else {
        return Ok(None);
    };
    let next_group_code = command
        .group_code
        .as_deref()
        .unwrap_or(current.group_code.as_str())
        .to_owned();
    let next_selection_mode = command
        .selection_mode
        .as_deref()
        .unwrap_or(current.selection_mode.as_str());
    let next_dynamic = is_dynamic_group(&next_group_code, next_selection_mode);
    if next_dynamic
        && command
            .members
            .as_ref()
            .is_some_and(|members| !members.is_empty())
    {
        return Err(DomainError::conflict(
            "dynamic API groups cannot maintain resource relationships",
        ));
    }
    if current.group_code == "api.all" && command.group_code.is_some() {
        return Err(DomainError::conflict(
            "api.all group code cannot be changed",
        ));
    }
    update_ai_resource_group_core(&mut tx, &command).await?;
    if next_group_code != current.group_code {
        rename_group_members_for_group_code(
            &mut tx,
            command.group_id,
            &current.group_code,
            &next_group_code,
            command.subject.tenant_id,
            command.subject.organization_id,
            &command.requested_at,
        )
        .await?;
    }
    if next_dynamic {
        replace_group_members_for_update(&mut tx, &next_group_code, &[], &command).await?;
    } else if let Some(members) = command.members.as_ref() {
        replace_group_members_for_update(&mut tx, &next_group_code, members, &command).await?;
    }
    insert_audit_log(
        &mut tx,
        &command.audit_log_uuid,
        &command.request_id,
        command.subject.tenant_id,
        command.subject.organization_id,
        command.subject.operator_id,
        command.subject.operator_type,
        "update_ai_resource_group",
        command.group_id,
        serde_json::json!({
            "action": "update_ai_resource_group",
            "groupId": command.group_id,
            "groupCodeChanged": command.group_code.is_some(),
            "membersChanged": command.members.is_some()
        }),
    )
    .await?;
    record_sqlite_ai_routing_config_change(
        &mut tx,
        ai_resource_group_routing_config_change(
            command.subject.tenant_id,
            command.subject.organization_id,
            command.subject.operator_id,
            &command.request_id,
            &command.requested_at,
            "update_ai_resource_group",
            command.group_id,
            serde_json::json!({
                "groupId": command.group_id,
                "groupCodeChanged": command.group_code.is_some(),
                "membersChanged": command.members.is_some()
            }),
        ),
    )
    .await?;
    let item = load_group_by_id(
        &mut tx,
        command.group_id,
        command.subject.tenant_id,
        command.subject.organization_id,
    )
    .await?;
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource group update transaction",
            error,
        )
    })?;
    Ok(item)
}

async fn upsert_ai_resource_group_member(
    pool: &SqlitePool,
    command: UpsertAdminAiResourceGroupMemberCommand,
) -> DomainResult<Option<AdminAiResourceGroupResourceItem>> {
    let mut tx = pool.begin().await.map_err(|error| {
        store_error(
            "failed to begin AI resource group member upsert transaction",
            error,
        )
    })?;
    let lock_result = sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET updated_at = updated_at
        WHERE id = ?
          AND tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
          AND COALESCE(NULLIF(group_type, ''), 'api_group') = 'api_group'
        "#,
    )
    .bind(command.group_id)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| store_error("failed to lock AI resource group", error))?;
    if lock_result.rows_affected() == 0 {
        return Ok(None);
    }
    let group = load_group_by_id(
        &mut tx,
        command.group_id,
        command.subject.tenant_id,
        command.subject.organization_id,
    )
    .await?
    .ok_or_else(|| DomainError::not_found("AI resource group was not found"))?;
    if is_dynamic_group(&group.group_code, &group.selection_mode) {
        return Err(DomainError::conflict(
            "dynamic API groups cannot maintain resource relationships",
        ));
    }

    let existing_member: i64 = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM ai_resource_group_item
            WHERE tenant_id = ?
              AND organization_id = ?
              AND resource_group_id = ?
              AND item_type = 'resource'
              AND resource_code = ?
              AND deleted_at IS NULL
              AND status = 1
        )
        "#,
    )
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(command.group_id)
    .bind(&command.member.resource_code)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| store_error("failed to inspect AI resource group member", error))?;
    if existing_member == 0 {
        let member_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM ai_resource_group_item
            WHERE tenant_id = ?
              AND organization_id = ?
              AND resource_group_id = ?
              AND item_type = 'resource'
              AND deleted_at IS NULL
              AND status = 1
            "#,
        )
        .bind(command.subject.tenant_id)
        .bind(command.subject.organization_id)
        .bind(command.group_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| store_error("failed to count AI resource group members", error))?;
        if member_count >= MAX_RESOURCE_GROUP_MEMBERS {
            return Err(DomainError::conflict(format!(
                "AI resource groups support at most {MAX_RESOURCE_GROUP_MEMBERS} members"
            )));
        }
    }

    let member_uuids = [command.member_uuid.clone()];
    let members = [command.member.clone()];
    insert_group_resource_members(
        &mut tx,
        command.group_id,
        &group.group_code,
        command.subject.tenant_id,
        command.subject.organization_id,
        &command.requested_at,
        &member_uuids,
        &members,
    )
    .await?;
    let item = load_group_resource_member(
        &mut tx,
        command.subject.tenant_id,
        command.subject.organization_id,
        command.group_id,
        &command.member.resource_code,
    )
    .await?
    .ok_or_else(|| DomainError::new("upserted AI resource group member could not be reloaded"))?;
    insert_audit_log(
        &mut tx,
        &command.audit_log_uuid,
        &command.request_id,
        command.subject.tenant_id,
        command.subject.organization_id,
        command.subject.operator_id,
        command.subject.operator_type,
        "upsert_ai_resource_group_member",
        command.group_id,
        serde_json::json!({
            "resourceCode": command.member.resource_code,
            "itemRole": command.member.item_role,
            "sortOrder": command.member.sort_order,
        }),
    )
    .await?;
    record_sqlite_ai_routing_config_change(
        &mut tx,
        ai_resource_group_routing_config_change(
            command.subject.tenant_id,
            command.subject.organization_id,
            command.subject.operator_id,
            &command.request_id,
            &command.requested_at,
            "upsert_ai_resource_group_member",
            command.group_id,
            serde_json::json!({
                "groupId": command.group_id,
                "resourceCode": &command.member.resource_code,
                "itemRole": &command.member.item_role,
                "sortOrder": command.member.sort_order,
            }),
        ),
    )
    .await?;
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource group member upsert transaction",
            error,
        )
    })?;
    Ok(Some(item))
}

async fn delete_ai_resource_group_member(
    pool: &SqlitePool,
    command: DeleteAdminAiResourceGroupMemberCommand,
) -> DomainResult<bool> {
    let mut tx = pool.begin().await.map_err(|error| {
        store_error(
            "failed to begin AI resource group member delete transaction",
            error,
        )
    })?;
    let lock_result = sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET updated_at = updated_at
        WHERE id = ?
          AND tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
          AND COALESCE(NULLIF(group_type, ''), 'api_group') = 'api_group'
        "#,
    )
    .bind(command.group_id)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| store_error("failed to lock AI resource group", error))?;
    if lock_result.rows_affected() == 0 {
        return Ok(false);
    }
    let group = load_group_by_id(
        &mut tx,
        command.group_id,
        command.subject.tenant_id,
        command.subject.organization_id,
    )
    .await?
    .ok_or_else(|| DomainError::not_found("AI resource group was not found"))?;
    if is_dynamic_group(&group.group_code, &group.selection_mode) {
        return Err(DomainError::conflict(
            "dynamic API groups cannot maintain resource relationships",
        ));
    }
    let result = sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = -1,
            deleted_at = ?,
            deleted_by = ?,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_id = ?
          AND item_type = 'resource'
          AND resource_code = ?
          AND deleted_at IS NULL
          AND status = 1
        "#,
    )
    .bind(&command.requested_at)
    .bind(command.subject.operator_id)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(command.group_id)
    .bind(&command.resource_code)
    .execute(&mut *tx)
    .await
    .map_err(|error| store_error("failed to delete AI resource group member", error))?;
    if result.rows_affected() > 0 {
        insert_audit_log(
            &mut tx,
            &command.audit_log_uuid,
            &command.request_id,
            command.subject.tenant_id,
            command.subject.organization_id,
            command.subject.operator_id,
            command.subject.operator_type,
            "delete_ai_resource_group_member",
            command.group_id,
            serde_json::json!({ "resourceCode": command.resource_code }),
        )
        .await?;
        record_sqlite_ai_routing_config_change(
            &mut tx,
            ai_resource_group_routing_config_change(
                command.subject.tenant_id,
                command.subject.organization_id,
                command.subject.operator_id,
                &command.request_id,
                &command.requested_at,
                "delete_ai_resource_group_member",
                command.group_id,
                serde_json::json!({
                    "groupId": command.group_id,
                    "resourceCode": &command.resource_code
                }),
            ),
        )
        .await?;
    }
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource group member delete transaction",
            error,
        )
    })?;
    Ok(true)
}

async fn delete_ai_resource_group(
    pool: &SqlitePool,
    command: DeleteAdminAiResourceGroupCommand,
) -> DomainResult<bool> {
    let mut tx = pool.begin().await.map_err(|error| {
        store_error(
            "failed to begin AI resource group delete transaction",
            error,
        )
    })?;
    let Some(current) = load_group_by_id(
        &mut tx,
        command.group_id,
        command.subject.tenant_id,
        command.subject.organization_id,
    )
    .await?
    else {
        return Ok(false);
    };
    if current.group_code == "api.all" || current.selection_mode == "dynamic_all_api" {
        return Err(DomainError::conflict(
            "dynamic API groups cannot be deleted",
        ));
    }
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = -1,
            deleted_at = ?,
            deleted_by = ?,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(&command.requested_at)
    .bind(command.subject.operator_id)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(command.group_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| store_error("failed to delete AI resource group items", error))?;
    let result = sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET status = -1,
            deleted_at = ?,
            deleted_by = ?,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE id = ?
          AND tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(&command.requested_at)
    .bind(command.subject.operator_id)
    .bind(&command.requested_at)
    .bind(command.group_id)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| store_error("failed to delete AI resource group", error))?;
    let deleted = result.rows_affected() > 0;
    if deleted {
        insert_audit_log(
            &mut tx,
            &command.audit_log_uuid,
            &command.request_id,
            command.subject.tenant_id,
            command.subject.organization_id,
            command.subject.operator_id,
            command.subject.operator_type,
            "delete_ai_resource_group",
            command.group_id,
            serde_json::json!({
                "action": "delete_ai_resource_group",
                "groupId": command.group_id
            }),
        )
        .await?;
        record_sqlite_ai_routing_config_change(
            &mut tx,
            ai_resource_group_routing_config_change(
                command.subject.tenant_id,
                command.subject.organization_id,
                command.subject.operator_id,
                &command.request_id,
                &command.requested_at,
                "delete_ai_resource_group",
                command.group_id,
                serde_json::json!({
                    "groupId": command.group_id
                }),
            ),
        )
        .await?;
    }
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource group delete transaction",
            error,
        )
    })?;
    Ok(deleted)
}

fn ai_resource_schema_for_create(command: &CreateAdminAiResourceCommand) -> String {
    let mut schema = serde_json::Map::new();
    schema.insert(
        "compositionMode".to_owned(),
        serde_json::Value::String(command.composition_mode.clone()),
    );
    if let Some(value) = command.access_channel_kind.as_ref() {
        schema.insert(
            "accessChannelKind".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(value) = command.base_url.as_ref() {
        schema.insert(
            "baseUrl".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(value) = command.default_vendor_code.as_ref() {
        schema.insert(
            "defaultVendorCode".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(value) = command.default_model_id.as_ref() {
        schema.insert(
            "defaultModelId".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if !command.supported_agent_provider_ids.is_empty() {
        schema.insert(
            "supportedAgentProviderIds".to_owned(),
            serde_json::json!(&command.supported_agent_provider_ids),
        );
    }
    if let Some(value) = command.description.as_ref() {
        schema.insert(
            "description".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    serde_json::Value::Object(schema).to_string()
}

fn ai_resource_schema_patch_for_update(command: &UpdateAdminAiResourceCommand) -> Option<String> {
    let mut patch = serde_json::Map::new();
    if let Some(value) = command.composition_mode.as_ref() {
        patch.insert(
            "compositionMode".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(value) = command.access_channel_kind.as_ref() {
        patch.insert(
            "accessChannelKind".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(value) = command.base_url.as_ref() {
        patch.insert(
            "baseUrl".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(value) = command.default_vendor_code.as_ref() {
        patch.insert(
            "defaultVendorCode".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(value) = command.default_model_id.as_ref() {
        patch.insert(
            "defaultModelId".to_owned(),
            serde_json::Value::String(value.clone()),
        );
    }
    if let Some(values) = command.supported_agent_provider_ids.as_ref() {
        patch.insert(
            "supportedAgentProviderIds".to_owned(),
            serde_json::json!(values),
        );
    }
    if let Some(value) = command.description.as_ref() {
        patch.insert(
            "description".to_owned(),
            value
                .as_ref()
                .map(|value| serde_json::Value::String(value.clone()))
                .unwrap_or(serde_json::Value::Null),
        );
    }
    (!patch.is_empty()).then(|| serde_json::Value::Object(patch).to_string())
}

async fn upsert_hierarchy_resource(
    tx: &mut Transaction<'_, Sqlite>,
    command: &ReplaceAdminAiResourceHierarchyCommand,
    node: &AdminAiResourceHierarchyNodeCommand,
) -> DomainResult<i64> {
    sqlx::query_scalar(
        r#"
        INSERT INTO ai_resource
            (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, resource_code, resource_type, display_name, vendor_code, modality_code, api_code, catalog_key, model, provider_native_model, resource_schema, description, sort_order, id)
        VALUES
            (?, ?, ?, 1, ?, ?, ?, 0, '{}', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(tenant_id, organization_id, resource_code) DO UPDATE SET
            status = excluded.status,
            updated_at = excluded.updated_at,
            version = COALESCE(ai_resource.version, 0) + 1,
            deleted_at = NULL,
            deleted_by = NULL,
            resource_type = excluded.resource_type,
            display_name = excluded.display_name,
            vendor_code = excluded.vendor_code,
            modality_code = excluded.modality_code,
            api_code = excluded.api_code,
            catalog_key = excluded.catalog_key,
            model = excluded.model,
            provider_native_model = excluded.provider_native_model,
            resource_schema = excluded.resource_schema,
            description = excluded.description,
            sort_order = COALESCE(excluded.sort_order, ai_resource.sort_order)
        RETURNING id
        "#,
    )
    .bind(&node.resource_uuid)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(status_code(&node.status))
    .bind(&command.requested_at)
    .bind(&command.requested_at)
    .bind(&node.resource_code)
    .bind(&node.resource_type)
    .bind(&node.display_name)
    .bind(node.vendor_code.as_deref())
    .bind(node.modality_code.as_deref())
    .bind(node.api_endpoint_code.as_deref())
    .bind(node.catalog_key.as_deref())
    .bind(node.model.as_deref())
    .bind(node.provider_native_model.as_deref())
    .bind(hierarchy_node_schema(node))
    .bind(node.description.as_deref())
    .bind(node.sort_order)
    .bind(next_claw_runtime_id("ai_resource")?)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| store_error("failed to upsert AI resource hierarchy node", error))
}

async fn replace_hierarchy_node_members(
    tx: &mut Transaction<'_, Sqlite>,
    command: &ReplaceAdminAiResourceHierarchyCommand,
    node: &AdminAiResourceHierarchyNodeCommand,
    resource_id: i64,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = -1, deleted_at = ?, deleted_by = ?, updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_code = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(&command.requested_at)
    .bind(command.subject.operator_id)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(&node.resource_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to clear AI resource hierarchy members", error))?;

    if node.members.is_empty() {
        sqlx::query(
            r#"
            UPDATE ai_resource_group
            SET status = -1, deleted_at = ?, deleted_by = ?, updated_at = ?,
                version = COALESCE(version, 0) + 1
            WHERE tenant_id = ?
              AND organization_id = ?
              AND group_code = ?
              AND deleted_at IS NULL
            "#,
        )
        .bind(&command.requested_at)
        .bind(command.subject.operator_id)
        .bind(&command.requested_at)
        .bind(command.subject.tenant_id)
        .bind(command.subject.organization_id)
        .bind(&node.resource_code)
        .execute(&mut **tx)
        .await
        .map_err(|error| {
            store_error("failed to retire empty AI resource hierarchy group", error)
        })?;
        return Ok(());
    }

    insert_members(
        tx,
        resource_id,
        &node.resource_code,
        command.subject.tenant_id,
        command.subject.organization_id,
        &command.requested_at,
        &node.member_uuids,
        &node.members,
    )
    .await?;
    sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET group_name = ?,
            group_type = ?,
            selection_mode = ?,
            status = ?,
            deleted_at = NULL,
            deleted_by = NULL,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND group_code = ?
        "#,
    )
    .bind(&node.display_name)
    .bind(&node.resource_type)
    .bind(&node.composition_mode)
    .bind(status_code(&node.status))
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(&node.resource_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to sync AI resource hierarchy group", error))?;
    Ok(())
}

async fn retire_stale_hierarchy_resources(
    tx: &mut Transaction<'_, Sqlite>,
    command: &ReplaceAdminAiResourceHierarchyCommand,
    desired_resource_codes: &HashSet<String>,
) -> DomainResult<usize> {
    let descendant_prefix = format!("{}.", command.root_resource_code);
    let rows = sqlx::query(
        r#"
        SELECT resource_code
        FROM ai_resource
        WHERE tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
          AND instr(resource_code, ?) = 1
        "#,
    )
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(descendant_prefix)
    .fetch_all(&mut **tx)
    .await
    .map_err(|error| store_error("failed to find stale AI resource hierarchy nodes", error))?;
    let stale_resource_codes = rows
        .into_iter()
        .map(|row| row.try_get::<String, _>("resource_code").map_err(row_error))
        .collect::<DomainResult<Vec<_>>>()?
        .into_iter()
        .filter(|resource_code| {
            resource_code_is_owned(command, resource_code)
                && !desired_resource_codes.contains(resource_code)
        })
        .collect::<Vec<_>>();

    for resource_code in &stale_resource_codes {
        sqlx::query(
            r#"
            UPDATE ai_resource_group_item
            SET status = -1, deleted_at = ?, deleted_by = ?, updated_at = ?,
                version = COALESCE(version, 0) + 1
            WHERE tenant_id = ?
              AND organization_id = ?
              AND (resource_group_code = ? OR resource_code = ?)
              AND deleted_at IS NULL
            "#,
        )
        .bind(&command.requested_at)
        .bind(command.subject.operator_id)
        .bind(&command.requested_at)
        .bind(command.subject.tenant_id)
        .bind(command.subject.organization_id)
        .bind(resource_code)
        .bind(resource_code)
        .execute(&mut **tx)
        .await
        .map_err(|error| store_error("failed to retire stale hierarchy members", error))?;
        sqlx::query(
            r#"
            UPDATE ai_resource_group
            SET status = -1, deleted_at = ?, deleted_by = ?, updated_at = ?,
                version = COALESCE(version, 0) + 1
            WHERE tenant_id = ?
              AND organization_id = ?
              AND group_code = ?
              AND deleted_at IS NULL
            "#,
        )
        .bind(&command.requested_at)
        .bind(command.subject.operator_id)
        .bind(&command.requested_at)
        .bind(command.subject.tenant_id)
        .bind(command.subject.organization_id)
        .bind(resource_code)
        .execute(&mut **tx)
        .await
        .map_err(|error| store_error("failed to retire stale hierarchy group", error))?;
        sqlx::query(
            r#"
            UPDATE ai_resource
            SET status = -1, deleted_at = ?, deleted_by = ?, updated_at = ?,
                version = COALESCE(version, 0) + 1
            WHERE tenant_id = ?
              AND organization_id = ?
              AND resource_code = ?
              AND deleted_at IS NULL
            "#,
        )
        .bind(&command.requested_at)
        .bind(command.subject.operator_id)
        .bind(&command.requested_at)
        .bind(command.subject.tenant_id)
        .bind(command.subject.organization_id)
        .bind(resource_code)
        .execute(&mut **tx)
        .await
        .map_err(|error| store_error("failed to retire stale hierarchy resource", error))?;
    }
    Ok(stale_resource_codes.len())
}

async fn insert_ai_resource(
    tx: &mut Transaction<'_, Sqlite>,
    command: &CreateAdminAiResourceCommand,
) -> DomainResult<i64> {
    let resource_id = next_claw_runtime_id("ai_resource")?;
    sqlx::query(
        r#"
        INSERT INTO ai_resource
            (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, resource_code, resource_type, display_name, vendor_code, modality_code, api_code, catalog_key, model, provider_native_model, resource_schema, description, sort_order, id)
        VALUES
            (?, ?, ?, 1, ?, ?, ?, 0, '{}', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&command.resource_uuid)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(status_code(&command.status))
    .bind(&command.requested_at)
    .bind(&command.requested_at)
    .bind(&command.resource_code)
    .bind(&command.resource_type)
    .bind(&command.display_name)
    .bind(command.vendor_code.as_deref())
    .bind(command.modality_code.as_deref())
    .bind(command.api_endpoint_code.as_deref())
    .bind(command.catalog_key.as_deref())
    .bind(command.model.as_deref())
    .bind(command.provider_native_model.as_deref())
    .bind(ai_resource_schema_for_create(command))
    .bind(command.description.as_deref())
    .bind(command.sort_order)
    .bind(resource_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create AI resource", error))?;
    Ok(resource_id)
}

async fn insert_ai_resource_group(
    tx: &mut Transaction<'_, Sqlite>,
    command: &CreateAdminAiResourceGroupCommand,
) -> DomainResult<i64> {
    let group_id = next_claw_runtime_id("ai_resource_group")?;
    sqlx::query(
        r#"
        INSERT INTO ai_resource_group
            (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, group_code, group_name, group_type, selection_mode, description, sort_order, id)
        VALUES
            (?, ?, ?, 1, ?, ?, ?, 0, '{}', ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&command.group_uuid)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(status_code(&command.status))
    .bind(&command.requested_at)
    .bind(&command.requested_at)
    .bind(&command.group_code)
    .bind(&command.group_name)
    .bind(&command.group_type)
    .bind(&command.selection_mode)
    .bind(command.description.as_deref())
    .bind(command.sort_order)
    .bind(group_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create AI resource group", error))?;
    Ok(group_id)
}

async fn update_ai_resource_core(
    tx: &mut Transaction<'_, Sqlite>,
    command: &UpdateAdminAiResourceCommand,
) -> DomainResult<()> {
    let resource_schema_patch = ai_resource_schema_patch_for_update(command);
    sqlx::query(
        r#"
        UPDATE ai_resource
        SET resource_code = COALESCE(?, resource_code),
            resource_type = COALESCE(?, resource_type),
            display_name = COALESCE(?, display_name),
            vendor_code = CASE WHEN ? THEN ? ELSE vendor_code END,
            modality_code = CASE WHEN ? THEN ? ELSE modality_code END,
            api_code = CASE WHEN ? THEN ? ELSE api_code END,
            catalog_key = CASE WHEN ? THEN ? ELSE catalog_key END,
            model = CASE WHEN ? THEN ? ELSE model END,
            provider_native_model = CASE WHEN ? THEN ? ELSE provider_native_model END,
            resource_schema = CASE
                WHEN ? IS NULL THEN resource_schema
                ELSE json_patch(COALESCE(resource_schema, '{}'), ?)
            END,
            description = CASE WHEN ? THEN ? ELSE description END,
            status = COALESCE(?, status),
            sort_order = CASE WHEN ? THEN ? ELSE sort_order END,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE id = ?
          AND tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(command.resource_code.as_deref())
    .bind(command.resource_type.as_deref())
    .bind(command.display_name.as_deref())
    .bind(present_flag(command.vendor_code.is_some()))
    .bind(optional_optional_str(&command.vendor_code))
    .bind(present_flag(command.modality_code.is_some()))
    .bind(optional_optional_str(&command.modality_code))
    .bind(present_flag(command.api_endpoint_code.is_some()))
    .bind(optional_optional_str(&command.api_endpoint_code))
    .bind(present_flag(command.catalog_key.is_some()))
    .bind(optional_optional_str(&command.catalog_key))
    .bind(present_flag(command.model.is_some()))
    .bind(optional_optional_str(&command.model))
    .bind(present_flag(command.provider_native_model.is_some()))
    .bind(optional_optional_str(&command.provider_native_model))
    .bind(resource_schema_patch.as_deref())
    .bind(resource_schema_patch.as_deref())
    .bind(present_flag(command.description.is_some()))
    .bind(optional_optional_str(&command.description))
    .bind(command.status.as_ref().map(|value| status_code(value)))
    .bind(present_flag(command.sort_order.is_some()))
    .bind(command.sort_order.flatten())
    .bind(&command.requested_at)
    .bind(command.resource_id)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to update AI resource", error))?;
    Ok(())
}

async fn update_ai_resource_group_core(
    tx: &mut Transaction<'_, Sqlite>,
    command: &UpdateAdminAiResourceGroupCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET group_code = COALESCE(?, group_code),
            group_name = COALESCE(?, group_name),
            group_type = COALESCE(?, group_type),
            selection_mode = COALESCE(?, selection_mode),
            description = CASE WHEN ? = 1 THEN ? ELSE description END,
            sort_order = CASE WHEN ? = 1 THEN ? ELSE sort_order END,
            status = COALESCE(?, status),
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE id = ?
          AND tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(command.group_code.as_deref())
    .bind(command.group_name.as_deref())
    .bind(command.group_type.as_deref())
    .bind(command.selection_mode.as_deref())
    .bind(present_flag(command.description.is_some()))
    .bind(optional_optional_str(&command.description))
    .bind(present_flag(command.sort_order.is_some()))
    .bind(command.sort_order.flatten())
    .bind(command.status.as_ref().map(|value| status_code(value)))
    .bind(&command.requested_at)
    .bind(command.group_id)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to update AI resource group", error))?;
    Ok(())
}

async fn replace_members_for_create(
    tx: &mut Transaction<'_, Sqlite>,
    resource_id: i64,
    command: &CreateAdminAiResourceCommand,
) -> DomainResult<()> {
    insert_members(
        tx,
        resource_id,
        &command.resource_code,
        command.subject.tenant_id,
        command.subject.organization_id,
        &command.requested_at,
        &command.member_uuids,
        &command.members,
    )
    .await
}

async fn replace_members_for_update(
    tx: &mut Transaction<'_, Sqlite>,
    previous_parent_resource_code: &str,
    effective_parent_resource_code: &str,
    members: &[AdminAiResourceMemberCommand],
    command: &UpdateAdminAiResourceCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = -1, deleted_at = ?, deleted_by = ?, updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_code = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(&command.requested_at)
    .bind(command.subject.operator_id)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(previous_parent_resource_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to clear AI resource members", error))?;

    insert_members(
        tx,
        command.resource_id,
        effective_parent_resource_code,
        command.subject.tenant_id,
        command.subject.organization_id,
        &command.requested_at,
        &command.member_uuids,
        members,
    )
    .await
}

async fn rename_members_for_resource_code(
    tx: &mut Transaction<'_, Sqlite>,
    _parent_resource_id: i64,
    previous_parent_resource_code: &str,
    effective_parent_resource_code: &str,
    command: &UpdateAdminAiResourceCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET resource_group_code = ?, updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_code = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(effective_parent_resource_code)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(previous_parent_resource_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to rename AI resource members", error))?;
    sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET group_code = ?, updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND group_code = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(effective_parent_resource_code)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(previous_parent_resource_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to rename resource group", error))?;
    Ok(())
}

async fn sync_resource_group_status(
    tx: &mut Transaction<'_, Sqlite>,
    resource_code: &str,
    command: &UpdateAdminAiResourceCommand,
    status: i32,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET status = ?,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND group_code = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(status)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(resource_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to sync resource group status", error))?;

    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = ?,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_code = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(status)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(resource_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to sync resource group member status", error))?;
    Ok(())
}

async fn insert_members(
    tx: &mut Transaction<'_, Sqlite>,
    parent_resource_id: i64,
    parent_resource_code: &str,
    tenant_id: i64,
    organization_id: i64,
    requested_at: &str,
    member_uuids: &[String],
    members: &[AdminAiResourceMemberCommand],
) -> DomainResult<()> {
    if members.is_empty() {
        return Ok(());
    }
    let group_id = ensure_resource_group(
        tx,
        parent_resource_id,
        parent_resource_code,
        tenant_id,
        organization_id,
        requested_at,
        "all",
    )
    .await?;
    for (index, member) in members.iter().enumerate() {
        let uuid = member_uuids
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("{parent_resource_code}-member-{index}"));
        let resolved_member = resolve_ai_resource_group_member(
            tx,
            tenant_id,
            organization_id,
            &member.member_resource_code,
        )
        .await?;
        sqlx::query(
            r#"
            INSERT INTO ai_resource_group_item
                (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, resource_group_id, resource_group_code, item_type, resource_id, resource_code, child_resource_group_id, child_resource_group_code, item_role, sort_order, id)
            VALUES
                (?, ?, ?, 1, 1, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(tenant_id, organization_id, resource_group_id, item_type, resource_code, child_resource_group_code) DO UPDATE SET
                status = 1,
                deleted_at = NULL,
                deleted_by = NULL,
                updated_at = excluded.updated_at,
                metadata = excluded.metadata,
                resource_id = excluded.resource_id,
                item_role = excluded.item_role,
                sort_order = excluded.sort_order,
                version = COALESCE(ai_resource_group_item.version, 0) + 1
            "#,
        )
        .bind(uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(requested_at)
        .bind(requested_at)
        .bind(serde_json::json!({ "required": member.required }).to_string())
        .bind(group_id)
        .bind(parent_resource_code)
        .bind(resolved_member.item_type)
        .bind(resolved_member.resource_id)
        .bind(&resolved_member.resource_code)
        .bind(resolved_member.child_resource_group_id)
        .bind(&resolved_member.child_resource_group_code)
        .bind(&member.member_role)
        .bind(member.sort_order)
        .bind(next_claw_runtime_id("ai_resource_group_item")?)
        .execute(&mut **tx)
        .await
        .map_err(|error| store_error("failed to upsert AI resource member", error))?;
    }
    Ok(())
}

async fn resolve_ai_resource_group_member(
    tx: &mut Transaction<'_, Sqlite>,
    tenant_id: i64,
    organization_id: i64,
    member_resource_code: &str,
) -> DomainResult<ResolvedAiResourceGroupMember> {
    if let Some(resource_group_id) = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM ai_resource_group
        WHERE tenant_id = ?
          AND organization_id = ?
          AND group_code = ?
          AND deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(member_resource_code)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to resolve AI resource member group", error))?
    {
        return Ok(ResolvedAiResourceGroupMember {
            item_type: "resource_group",
            resource_id: None,
            resource_code: String::new(),
            child_resource_group_id: Some(resource_group_id),
            child_resource_group_code: member_resource_code.to_owned(),
        });
    }

    let resource_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM ai_resource
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_code = ?
          AND deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(member_resource_code)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to resolve AI resource member", error))?
    .ok_or_else(|| {
        DomainError::not_found(format!(
            "AI resource member was not found: {member_resource_code}"
        ))
    })?;

    Ok(ResolvedAiResourceGroupMember {
        item_type: "resource",
        resource_id: Some(resource_id),
        resource_code: member_resource_code.to_owned(),
        child_resource_group_id: None,
        child_resource_group_code: String::new(),
    })
}

async fn ensure_resource_group(
    tx: &mut Transaction<'_, Sqlite>,
    resource_id: i64,
    resource_code: &str,
    tenant_id: i64,
    organization_id: i64,
    requested_at: &str,
    composition_mode: &str,
) -> DomainResult<i64> {
    let group_uuid = format!("ai-resource-group-{resource_id}");
    if let Some(group_id) = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM ai_resource_group
        WHERE tenant_id = ?
          AND organization_id = ?
          AND group_code = ?
          AND deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(resource_code)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load resource group", error))?
    {
        return Ok(group_id);
    }

    sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET group_code = ?,
            group_name = COALESCE((
                SELECT NULLIF(display_name, '')
                FROM ai_resource r
                WHERE r.id = ?
                  AND r.tenant_id = ai_resource_group.tenant_id
                  AND r.organization_id = ai_resource_group.organization_id
                  AND r.deleted_at IS NULL
                LIMIT 1
            ), ?),
            selection_mode = COALESCE(NULLIF(selection_mode, ''), ?),
            status = 1,
            deleted_at = NULL,
            deleted_by = NULL,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE uuid = ?
          AND tenant_id = ?
          AND organization_id = ?
        "#,
    )
    .bind(resource_code)
    .bind(resource_id)
    .bind(resource_code)
    .bind(composition_mode)
    .bind(requested_at)
    .bind(&group_uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to update resource group", error))?;
    if let Some(group_id) = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM ai_resource_group
        WHERE uuid = ?
          AND tenant_id = ?
          AND organization_id = ?
        LIMIT 1
        "#,
    )
    .bind(&group_uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to reload resource group", error))?
    {
        return Ok(group_id);
    }

    let group_id = next_claw_runtime_id("ai_resource_group")?;
    let result = sqlx::query(
        r#"
        INSERT INTO ai_resource_group
            (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, group_code, group_name, group_type, selection_mode, sort_order, id)
        SELECT
            ?,
            tenant_id,
            organization_id,
            data_scope,
            status,
            ?,
            ?,
            0,
            '{}',
            resource_code,
            COALESCE(NULLIF(display_name, ''), resource_code),
            resource_type,
            ?,
            sort_order,
            ?
        FROM ai_resource
        WHERE id = ?
          AND tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(group_uuid)
    .bind(requested_at)
    .bind(requested_at)
    .bind(composition_mode)
    .bind(group_id)
    .bind(resource_id)
    .bind(tenant_id)
    .bind(organization_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create resource group", error))?;
    if result.rows_affected() == 0 {
        return Err(DomainError::not_found(format!(
            "AI resource was not found: {resource_id}"
        )));
    }
    Ok(group_id)
}

async fn load_resource_by_id(
    tx: &mut Transaction<'_, Sqlite>,
    resource_id: i64,
    tenant_id: i64,
    organization_id: i64,
) -> DomainResult<Option<AdminAiResourceItem>> {
    let row = sqlx::query(
        r#"
        SELECT
            id,
            resource_code,
            resource_type AS resource_type,
            COALESCE(NULLIF(display_name, ''), resource_code) AS display_name,
            vendor_code,
            modality_code,
            api_code AS api_endpoint_code,
            catalog_key,
            model,
            provider_native_model,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.accessChannelKind'), '') AS access_channel_kind,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.baseUrl'), '') AS base_url,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.defaultVendorCode'), '') AS default_vendor_code,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.defaultModelId'), '') AS default_model_id,
            COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.supportedAgentProviderIds'), '[]') AS supported_agent_provider_ids_json,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.contextTokens') IN ('integer', 'real')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.contextTokens') AS INTEGER)
            END AS context_tokens,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.maxOutputTokens') IN ('integer', 'real')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.maxOutputTokens') AS INTEGER)
            END AS max_output_tokens,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.toolCallRounds') IN ('integer', 'real')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.toolCallRounds') AS INTEGER)
            END AS tool_call_rounds,
            CASE WHEN json_type(COALESCE(resource_schema, '{}'), '$.supportsMultimodal') IN ('true', 'false')
                THEN CAST(json_extract(COALESCE(resource_schema, '{}'), '$.supportsMultimodal') AS INTEGER)
            END AS supports_multimodal,
            COALESCE(
                NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.description'), ''),
                NULLIF(description, '')
            ) AS description,
            NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.capability'), '') AS capability,
            COALESCE(json_extract(COALESCE(resource_schema, '{}'), '$.capabilities'), '[]') AS capabilities_json,
            COALESCE(
                NULLIF(json_extract(COALESCE(resource_schema, '{}'), '$.compositionMode'), ''),
                (
                    SELECT NULLIF(g.selection_mode, '')
                    FROM ai_resource_group g
                    WHERE g.tenant_id = ai_resource.tenant_id
                      AND g.organization_id = ai_resource.organization_id
                      AND g.group_code = ai_resource.resource_code
                      AND g.deleted_at IS NULL
                    LIMIT 1
                ),
                'single'
            ) AS composition_mode,
            status,
            sort_order
        FROM ai_resource
        WHERE id = ?
          AND tenant_id = ?
          AND organization_id = ?
          AND deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(resource_id)
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load AI resource", error))?;

    let Some(row) = row else {
        return Ok(None);
    };
    let resource_code = row.try_get("resource_code").map_err(row_error)?;
    let members = load_members_tx(tx, tenant_id, organization_id, &[resource_code]).await?;
    item_from_row(row, &members).map(Some)
}

async fn load_members(
    pool: &SqlitePool,
    tenant_id: i64,
    organization_id: i64,
    parent_resource_codes: &[String],
) -> DomainResult<HashMap<String, Vec<AdminAiResourceMemberItem>>> {
    if parent_resource_codes.is_empty() {
        return Ok(HashMap::new());
    }
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            resource_group_code AS parent_resource_code,
            COALESCE(NULLIF(resource_code, ''), child_resource_group_code) AS member_resource_code,
            COALESCE(NULLIF(item_role, ''), 'included') AS member_role,
            COALESCE(json_extract(COALESCE(metadata, '{}'), '$.required'), 1) AS required,
            sort_order
        FROM ai_resource_group_item
        "#,
    );
    query
        .push(" WHERE tenant_id = ")
        .push_bind(tenant_id)
        .push(" AND organization_id = ")
        .push_bind(organization_id)
        .push(" AND resource_group_code IN (");
    let mut codes = query.separated(", ");
    for resource_code in parent_resource_codes {
        codes.push_bind(resource_code);
    }
    codes.push_unseparated(")");
    query.push(
        " AND deleted_at IS NULL AND status = 1 \
         ORDER BY resource_group_code ASC, COALESCE(sort_order, 100000) ASC, id ASC",
    );
    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(|error| store_error("failed to list AI resource members", error))?;

    members_from_rows(rows)
}

async fn load_members_tx(
    tx: &mut Transaction<'_, Sqlite>,
    tenant_id: i64,
    organization_id: i64,
    parent_resource_codes: &[String],
) -> DomainResult<HashMap<String, Vec<AdminAiResourceMemberItem>>> {
    if parent_resource_codes.is_empty() {
        return Ok(HashMap::new());
    }
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            resource_group_code AS parent_resource_code,
            COALESCE(NULLIF(resource_code, ''), child_resource_group_code) AS member_resource_code,
            COALESCE(NULLIF(item_role, ''), 'included') AS member_role,
            COALESCE(json_extract(COALESCE(metadata, '{}'), '$.required'), 1) AS required,
            sort_order
        FROM ai_resource_group_item
        "#,
    );
    query
        .push(" WHERE tenant_id = ")
        .push_bind(tenant_id)
        .push(" AND organization_id = ")
        .push_bind(organization_id)
        .push(" AND resource_group_code IN (");
    let mut codes = query.separated(", ");
    for resource_code in parent_resource_codes {
        codes.push_bind(resource_code);
    }
    codes.push_unseparated(")");
    query.push(
        " AND deleted_at IS NULL AND status = 1 \
         ORDER BY resource_group_code ASC, COALESCE(sort_order, 100000) ASC, id ASC",
    );
    let rows = query
        .build()
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| store_error("failed to list AI resource members", error))?;

    members_from_rows(rows)
}

fn members_from_rows(
    rows: Vec<sqlx::sqlite::SqliteRow>,
) -> DomainResult<HashMap<String, Vec<AdminAiResourceMemberItem>>> {
    let mut members = HashMap::<String, Vec<AdminAiResourceMemberItem>>::new();
    for row in rows {
        let parent_resource_code: String =
            row.try_get("parent_resource_code").map_err(row_error)?;
        members
            .entry(parent_resource_code.clone())
            .or_default()
            .push(AdminAiResourceMemberItem {
                parent_resource_code,
                member_resource_code: row.try_get("member_resource_code").map_err(row_error)?,
                member_role: row.try_get("member_role").map_err(row_error)?,
                required: row
                    .try_get::<i64, _>("required")
                    .map(|value| value != 0)
                    .map_err(row_error)?,
                sort_order: row.try_get("sort_order").ok().flatten(),
            });
    }
    Ok(members)
}

fn resource_search_pattern(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"))
}

fn item_from_row(
    row: sqlx::sqlite::SqliteRow,
    members: &HashMap<String, Vec<AdminAiResourceMemberItem>>,
) -> DomainResult<AdminAiResourceItem> {
    let resource_code: String = row.try_get("resource_code").map_err(row_error)?;
    let status: i64 = row.try_get("status").map_err(row_error)?;
    Ok(AdminAiResourceItem {
        id: row.try_get("id").map_err(row_error)?,
        resource_code: resource_code.clone(),
        resource_type: row.try_get("resource_type").map_err(row_error)?,
        display_name: row.try_get("display_name").map_err(row_error)?,
        vendor_code: optional_string_cell(&row, "vendor_code"),
        modality_code: optional_string_cell(&row, "modality_code"),
        api_endpoint_code: optional_string_cell(&row, "api_endpoint_code"),
        catalog_key: optional_string_cell(&row, "catalog_key"),
        model: optional_string_cell(&row, "model"),
        provider_native_model: optional_string_cell(&row, "provider_native_model"),
        access_channel_kind: optional_string_cell(&row, "access_channel_kind"),
        base_url: optional_string_cell(&row, "base_url"),
        default_vendor_code: optional_string_cell(&row, "default_vendor_code"),
        default_model_id: optional_string_cell(&row, "default_model_id"),
        supported_agent_provider_ids: string_array_cell(&row, "supported_agent_provider_ids_json")?,
        context_tokens: row.try_get("context_tokens").map_err(row_error)?,
        max_output_tokens: row.try_get("max_output_tokens").map_err(row_error)?,
        tool_call_rounds: row.try_get("tool_call_rounds").map_err(row_error)?,
        supports_multimodal: row
            .try_get::<Option<i64>, _>("supports_multimodal")
            .map_err(row_error)?
            .map(|value| value != 0),
        description: optional_string_cell(&row, "description"),
        capability: optional_string_cell(&row, "capability"),
        capabilities: string_array_cell(&row, "capabilities_json")?,
        composition_mode: row.try_get("composition_mode").map_err(row_error)?,
        status: status_label(status),
        sort_order: row.try_get("sort_order").ok().flatten(),
        members: members.get(&resource_code).cloned().unwrap_or_default(),
    })
}

fn group_item_from_row(row: sqlx::sqlite::SqliteRow) -> DomainResult<AdminAiResourceGroupItem> {
    let status: i64 = row.try_get("status").map_err(row_error)?;
    let dynamic = row
        .try_get::<i64, _>("dynamic")
        .map(|value| value != 0)
        .unwrap_or(false);
    Ok(AdminAiResourceGroupItem {
        id: row.try_get("id").map_err(row_error)?,
        group_code: row.try_get("group_code").map_err(row_error)?,
        group_name: row.try_get("group_name").map_err(row_error)?,
        group_type: row.try_get("group_type").map_err(row_error)?,
        selection_mode: row.try_get("selection_mode").map_err(row_error)?,
        description: optional_string_cell(&row, "description"),
        vendor_codes: string_array_cell_or_empty(&row, "vendor_codes_json")?,
        capability: optional_string_cell(&row, "capability"),
        capabilities: string_array_cell_or_empty(&row, "capabilities_json")?,
        sort_order: row.try_get("sort_order").ok().flatten(),
        status: status_label(status),
        resource_count: row.try_get("resource_count").unwrap_or(0),
        dynamic,
    })
}

#[derive(Default)]
struct AiResourceGroupSummary {
    vendor_codes: Vec<String>,
    capabilities: Vec<String>,
}

async fn hydrate_group_summaries(
    pool: &SqlitePool,
    tenant_id: i64,
    organization_id: i64,
    groups: &mut [AdminAiResourceGroupItem],
) -> DomainResult<()> {
    if groups.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query(
        r#"
        SELECT
            g.id AS group_id,
            r.vendor_code,
            COALESCE(
                NULLIF(json_extract(COALESCE(r.resource_schema, '{}'), '$.capability'), ''),
                NULLIF(r.modality_code, '')
            ) AS capability
        FROM ai_resource_group g
        JOIN ai_resource r
          ON (
                g.selection_mode = 'dynamic_all_api'
                AND r.resource_type = 'api_endpoint'
             )
             OR (
                COALESCE(NULLIF(g.selection_mode, ''), 'manual') <> 'dynamic_all_api'
                AND EXISTS (
                    SELECT 1
                    FROM ai_resource_group_item item
                    WHERE item.tenant_id = g.tenant_id
                      AND item.organization_id = g.organization_id
                      AND item.resource_group_id = g.id
                      AND item.item_type = 'resource'
                      AND item.resource_code = r.resource_code
                      AND item.deleted_at IS NULL
                      AND item.status = 1
                )
             )
        WHERE (
                (g.tenant_id = ? AND g.organization_id = ?)
                OR (g.tenant_id = 0 AND g.organization_id = 0)
              )
          AND g.deleted_at IS NULL
          AND COALESCE(NULLIF(g.group_type, ''), 'api_group') = 'api_group'
          AND NOT (
              g.tenant_id = 0
              AND g.organization_id = 0
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource_group tenant_group
                  WHERE tenant_group.tenant_id = ?
                    AND tenant_group.organization_id = ?
                    AND tenant_group.group_code = g.group_code
                    AND tenant_group.deleted_at IS NULL
                    AND COALESCE(NULLIF(tenant_group.group_type, ''), 'api_group') = 'api_group'
              )
          )
          AND (
                (r.tenant_id = g.tenant_id AND r.organization_id = g.organization_id)
                OR (r.tenant_id = 0 AND r.organization_id = 0)
              )
          AND r.deleted_at IS NULL
          AND NOT (
              r.tenant_id = 0
              AND r.organization_id = 0
              AND (g.tenant_id <> 0 OR g.organization_id <> 0)
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource tenant_resource
                  WHERE tenant_resource.tenant_id = g.tenant_id
                    AND tenant_resource.organization_id = g.organization_id
                    AND tenant_resource.resource_code = r.resource_code
                    AND tenant_resource.deleted_at IS NULL
              )
          )
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to summarize AI resource groups", error))?;

    let mut summaries = HashMap::<i64, AiResourceGroupSummary>::new();
    for row in rows {
        let group_id: i64 = row.try_get("group_id").map_err(row_error)?;
        let summary = summaries.entry(group_id).or_default();
        if let Some(vendor_code) = optional_string_cell(&row, "vendor_code") {
            push_unique_lowercase(&mut summary.vendor_codes, vendor_code);
        }
        if let Some(capability) = optional_string_cell(&row, "capability") {
            push_unique_lowercase(&mut summary.capabilities, capability);
        }
    }
    for group in groups {
        if let Some(summary) = summaries.remove(&group.id) {
            group.vendor_codes = summary.vendor_codes;
            group.capability = if summary.capabilities.len() == 1 {
                summary.capabilities.first().cloned()
            } else {
                None
            };
            group.capabilities = summary.capabilities;
        }
    }
    Ok(())
}

fn group_resource_from_row(
    row: sqlx::sqlite::SqliteRow,
) -> DomainResult<AdminAiResourceGroupResourceItem> {
    let status: i64 = row.try_get("status").map_err(row_error)?;
    Ok(AdminAiResourceGroupResourceItem {
        id: row.try_get("id").map_err(row_error)?,
        resource_code: row.try_get("resource_code").map_err(row_error)?,
        resource_type: row.try_get("resource_type").map_err(row_error)?,
        display_name: row.try_get("display_name").map_err(row_error)?,
        vendor_code: optional_string_cell(&row, "vendor_code"),
        modality_code: optional_string_cell(&row, "modality_code"),
        api_endpoint_code: optional_string_cell(&row, "api_endpoint_code"),
        catalog_key: optional_string_cell(&row, "catalog_key"),
        model: optional_string_cell(&row, "model"),
        provider_native_model: optional_string_cell(&row, "provider_native_model"),
        status: status_label(status),
        sort_order: row.try_get("sort_order").ok().flatten(),
        member_role: row.try_get("member_role").map_err(row_error)?,
    })
}

struct ResourceGroupHeader {
    id: i64,
    tenant_id: i64,
    organization_id: i64,
    group_code: String,
    selection_mode: String,
}

async fn load_group_header(
    pool: &SqlitePool,
    tenant_id: i64,
    organization_id: i64,
    group_id_or_code: &str,
) -> DomainResult<Option<ResourceGroupHeader>> {
    let numeric_id = group_id_or_code.trim().parse::<i64>().ok();
    let row = sqlx::query(
        r#"
        SELECT
            id,
            tenant_id,
            organization_id,
            group_code,
            COALESCE(NULLIF(selection_mode, ''), 'manual') AS selection_mode
        FROM ai_resource_group
        WHERE (
                (tenant_id = ? AND organization_id = ?)
                OR (tenant_id = 0 AND organization_id = 0)
              )
          AND deleted_at IS NULL
          AND COALESCE(NULLIF(group_type, ''), 'api_group') = 'api_group'
          AND (? IS NOT NULL AND id = ? OR group_code = ?)
        ORDER BY CASE WHEN ? IS NOT NULL AND id = ? THEN 0 ELSE 1 END,
                 CASE WHEN tenant_id = ? AND organization_id = ? THEN 0 ELSE 1 END,
                 id ASC
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(numeric_id)
    .bind(numeric_id.unwrap_or(0))
    .bind(group_id_or_code)
    .bind(numeric_id)
    .bind(numeric_id.unwrap_or(0))
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("failed to resolve AI resource group", error))?;
    row.map(|row| {
        Ok(ResourceGroupHeader {
            id: row.try_get("id").map_err(row_error)?,
            tenant_id: row.try_get("tenant_id").map_err(row_error)?,
            organization_id: row.try_get("organization_id").map_err(row_error)?,
            group_code: row.try_get("group_code").map_err(row_error)?,
            selection_mode: row.try_get("selection_mode").map_err(row_error)?,
        })
    })
    .transpose()
}

async fn load_group_by_id(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: i64,
    tenant_id: i64,
    organization_id: i64,
) -> DomainResult<Option<AdminAiResourceGroupItem>> {
    let row = sqlx::query(
        r#"
        SELECT
            g.id,
            g.group_code,
            g.group_name,
            COALESCE(NULLIF(g.group_type, ''), 'api_group') AS group_type,
            COALESCE(NULLIF(g.selection_mode, ''), 'manual') AS selection_mode,
            g.description,
            g.sort_order,
            g.status,
            CASE
                WHEN g.selection_mode = 'dynamic_all_api' THEN (
                    SELECT COUNT(1)
                    FROM ai_resource r
                    WHERE (
                            (r.tenant_id = g.tenant_id AND r.organization_id = g.organization_id)
                            OR (r.tenant_id = 0 AND r.organization_id = 0)
                          )
                      AND r.resource_type = 'api_endpoint'
                      AND r.deleted_at IS NULL
                      AND NOT (
                          r.tenant_id = 0
                          AND r.organization_id = 0
                          AND (g.tenant_id <> 0 OR g.organization_id <> 0)
                          AND EXISTS (
                              SELECT 1
                              FROM ai_resource tenant_resource
                              WHERE tenant_resource.tenant_id = g.tenant_id
                                AND tenant_resource.organization_id = g.organization_id
                                AND tenant_resource.resource_code = r.resource_code
                                AND tenant_resource.deleted_at IS NULL
                          )
                      )
                )
                ELSE (
                    SELECT COUNT(1)
                    FROM ai_resource_group_item item
                    JOIN ai_resource r
                      ON r.resource_code = item.resource_code
                     AND r.deleted_at IS NULL
                     AND (
                          (r.tenant_id = item.tenant_id AND r.organization_id = item.organization_id)
                          OR (r.tenant_id = 0 AND r.organization_id = 0)
                     )
                     AND NOT (
                          r.tenant_id = 0
                          AND r.organization_id = 0
                          AND (item.tenant_id <> 0 OR item.organization_id <> 0)
                          AND EXISTS (
                              SELECT 1
                              FROM ai_resource tenant_resource
                              WHERE tenant_resource.tenant_id = item.tenant_id
                                AND tenant_resource.organization_id = item.organization_id
                                AND tenant_resource.resource_code = item.resource_code
                                AND tenant_resource.deleted_at IS NULL
                          )
                     )
                    WHERE item.tenant_id = g.tenant_id
                      AND item.organization_id = g.organization_id
                      AND item.resource_group_id = g.id
                      AND item.item_type = 'resource'
                      AND item.deleted_at IS NULL
                      AND item.status = 1
                )
            END AS resource_count,
            CASE WHEN g.selection_mode = 'dynamic_all_api' THEN 1 ELSE 0 END AS dynamic
        FROM ai_resource_group g
        WHERE g.id = ?
          AND g.tenant_id = ?
          AND g.organization_id = ?
          AND g.deleted_at IS NULL
          AND COALESCE(NULLIF(g.group_type, ''), 'api_group') = 'api_group'
        LIMIT 1
        "#,
    )
    .bind(group_id)
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load AI resource group", error))?;
    row.map(group_item_from_row).transpose()
}

async fn replace_group_members_for_create(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: i64,
    command: &CreateAdminAiResourceGroupCommand,
) -> DomainResult<()> {
    if is_dynamic_group(&command.group_code, &command.selection_mode) {
        return Ok(());
    }
    insert_group_resource_members(
        tx,
        group_id,
        &command.group_code,
        command.subject.tenant_id,
        command.subject.organization_id,
        &command.requested_at,
        &command.member_uuids,
        &command.members,
    )
    .await
}

async fn replace_group_members_for_update(
    tx: &mut Transaction<'_, Sqlite>,
    effective_group_code: &str,
    members: &[sdkwork_models_contract_service::AdminAiResourceGroupMemberCommand],
    command: &UpdateAdminAiResourceGroupCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = -1,
            deleted_at = ?,
            deleted_by = ?,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(&command.requested_at)
    .bind(command.subject.operator_id)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(command.group_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to clear AI resource group members", error))?;

    insert_group_resource_members(
        tx,
        command.group_id,
        effective_group_code,
        command.subject.tenant_id,
        command.subject.organization_id,
        &command.requested_at,
        &command.member_uuids,
        members,
    )
    .await
}

async fn load_group_resource_member(
    tx: &mut Transaction<'_, Sqlite>,
    tenant_id: i64,
    organization_id: i64,
    group_id: i64,
    resource_code: &str,
) -> DomainResult<Option<AdminAiResourceGroupResourceItem>> {
    let row = sqlx::query(
        r#"
        SELECT
            r.id,
            r.resource_code,
            r.resource_type,
            COALESCE(NULLIF(r.display_name, ''), r.resource_code) AS display_name,
            r.vendor_code,
            r.modality_code,
            r.api_code AS api_endpoint_code,
            r.catalog_key,
            r.model,
            r.provider_native_model,
            r.status,
            COALESCE(item.sort_order, r.sort_order) AS sort_order,
            COALESCE(NULLIF(item.item_role, ''), 'included') AS member_role
        FROM ai_resource_group_item item
        JOIN ai_resource r
          ON r.resource_code = item.resource_code
         AND r.deleted_at IS NULL
         AND (
              (r.tenant_id = item.tenant_id AND r.organization_id = item.organization_id)
              OR (r.tenant_id = 0 AND r.organization_id = 0)
         )
         AND NOT (
              r.tenant_id = 0
              AND r.organization_id = 0
              AND (item.tenant_id <> 0 OR item.organization_id <> 0)
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource tenant_resource
                  WHERE tenant_resource.tenant_id = item.tenant_id
                    AND tenant_resource.organization_id = item.organization_id
                    AND tenant_resource.resource_code = item.resource_code
                    AND tenant_resource.deleted_at IS NULL
              )
         )
        WHERE item.tenant_id = ?
          AND item.organization_id = ?
          AND item.resource_group_id = ?
          AND item.item_type = 'resource'
          AND item.resource_code = ?
          AND item.deleted_at IS NULL
          AND item.status = 1
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(group_id)
    .bind(resource_code)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to reload AI resource group member", error))?;
    row.map(group_resource_from_row).transpose()
}

async fn insert_group_resource_members(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: i64,
    group_code: &str,
    tenant_id: i64,
    organization_id: i64,
    requested_at: &str,
    member_uuids: &[String],
    members: &[sdkwork_models_contract_service::AdminAiResourceGroupMemberCommand],
) -> DomainResult<()> {
    for (index, member) in members.iter().enumerate() {
        let uuid = member_uuids
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("{group_code}-member-{index}"));
        let resource_id = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM ai_resource
            WHERE (
                    (tenant_id = ? AND organization_id = ?)
                    OR (tenant_id = 0 AND organization_id = 0)
                  )
              AND resource_code = ?
              AND resource_type = 'api_endpoint'
              AND deleted_at IS NULL
            ORDER BY CASE WHEN tenant_id = ? AND organization_id = ? THEN 0 ELSE 1 END,
                     id ASC
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&member.resource_code)
        .bind(tenant_id)
        .bind(organization_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| store_error("failed to resolve AI resource group member", error))?
        .ok_or_else(|| {
            DomainError::not_found(format!(
                "AI API resource was not found: {}",
                member.resource_code
            ))
        })?;
        sqlx::query(
            r#"
            INSERT INTO ai_resource_group_item
                (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, resource_group_id, resource_group_code, item_type, resource_id, resource_code, child_resource_group_id, child_resource_group_code, item_role, sort_order, id)
            VALUES
                (?, ?, ?, 1, 1, ?, ?, 0, '{}', ?, ?, 'resource', ?, ?, NULL, '', ?, ?, ?)
            ON CONFLICT(tenant_id, organization_id, resource_group_id, item_type, resource_code, child_resource_group_code) DO UPDATE SET
                status = 1,
                deleted_at = NULL,
                deleted_by = NULL,
                updated_at = excluded.updated_at,
                resource_id = excluded.resource_id,
                item_role = excluded.item_role,
                sort_order = excluded.sort_order,
                version = COALESCE(ai_resource_group_item.version, 0) + 1
            "#,
        )
        .bind(uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(requested_at)
        .bind(requested_at)
        .bind(group_id)
        .bind(group_code)
        .bind(resource_id)
        .bind(&member.resource_code)
        .bind(&member.item_role)
        .bind(member.sort_order)
        .bind(next_claw_runtime_id("ai_resource_group_item")?)
        .execute(&mut **tx)
        .await
        .map_err(|error| store_error("failed to upsert AI resource group member", error))?;
    }
    Ok(())
}

async fn rename_group_members_for_group_code(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: i64,
    previous_group_code: &str,
    next_group_code: &str,
    tenant_id: i64,
    organization_id: i64,
    requested_at: &str,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET resource_group_code = ?,
            updated_at = ?,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = ?
          AND organization_id = ?
          AND resource_group_id = ?
          AND resource_group_code = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(next_group_code)
    .bind(requested_at)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(group_id)
    .bind(previous_group_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to rename AI resource group members", error))?;
    Ok(())
}

fn optional_string_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(name)
        .ok()
        .flatten()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn string_array_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> DomainResult<Vec<String>> {
    let raw = row
        .try_get::<Option<String>, _>(name)
        .map_err(row_error)?
        .unwrap_or_else(|| "[]".to_owned());
    parse_string_array_json(&raw, name)
}

fn string_array_cell_or_empty(
    row: &sqlx::sqlite::SqliteRow,
    name: &str,
) -> DomainResult<Vec<String>> {
    let raw = row
        .try_get::<Option<String>, _>(name)
        .ok()
        .flatten()
        .unwrap_or_else(|| "[]".to_owned());
    parse_string_array_json(&raw, name)
}

fn parse_string_array_json(raw: &str, name: &str) -> DomainResult<Vec<String>> {
    let value = raw.trim();
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: serde_json::Value = serde_json::from_str(value)
        .map_err(|error| DomainError::new(format!("invalid AI resource {name} json: {error}")))?;
    let Some(items) = parsed.as_array() else {
        return Ok(Vec::new());
    };
    Ok(items
        .iter()
        .filter_map(|item| item.as_str())
        .map(|item| item.trim().to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect())
}

fn push_unique_lowercase(values: &mut Vec<String>, value: String) {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() || values.iter().any(|current| current == &normalized) {
        return;
    }
    values.push(normalized);
}

fn status_label(status: i64) -> String {
    match status {
        1 => "active",
        0 => "disabled",
        _ => "inactive",
    }
    .to_owned()
}

fn status_code(status: &str) -> i32 {
    match status {
        "disabled" => 0,
        "inactive" => -1,
        _ => 1,
    }
}

fn present_flag(is_present: bool) -> i32 {
    if is_present {
        1
    } else {
        0
    }
}

fn optional_optional_str(value: &Option<Option<String>>) -> Option<&str> {
    value.as_ref().and_then(|inner| inner.as_deref())
}

fn is_dynamic_group(_group_code: &str, selection_mode: &str) -> bool {
    selection_mode == "dynamic_all_api"
}

fn ai_resource_routing_config_change<'a>(
    tenant_id: i64,
    organization_id: i64,
    operator_id: i64,
    request_id: &'a str,
    requested_at: &'a str,
    action: &'a str,
    resource_id: i64,
    event_payload: serde_json::Value,
) -> AiRoutingConfigChange<'a> {
    AiRoutingConfigChange {
        tenant_id,
        organization_id,
        operator_id,
        request_id,
        requested_at,
        changed_object_type: "ai_resource",
        changed_object_id: resource_id,
        action,
        event_payload,
    }
}

fn ai_resource_group_routing_config_change<'a>(
    tenant_id: i64,
    organization_id: i64,
    operator_id: i64,
    request_id: &'a str,
    requested_at: &'a str,
    action: &'a str,
    group_id: i64,
    event_payload: serde_json::Value,
) -> AiRoutingConfigChange<'a> {
    AiRoutingConfigChange {
        tenant_id,
        organization_id,
        operator_id,
        request_id,
        requested_at,
        changed_object_type: "ai_resource_group",
        changed_object_id: group_id,
        action,
        event_payload,
    }
}

async fn insert_audit_log(
    tx: &mut Transaction<'_, Sqlite>,
    audit_log_uuid: &str,
    request_id: &str,
    tenant_id: i64,
    organization_id: i64,
    operator_id: i64,
    operator_type: i32,
    action: &'static str,
    target_id: i64,
    change_summary: serde_json::Value,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        INSERT INTO ops_audit_log
            (id, uuid, tenant_id, organization_id, action, target_type, target_id, request_id, operator_id, operator_type, change_summary)
        VALUES
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(next_claw_runtime_id("ops_audit_log")?)
    .bind(audit_log_uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(action)
    .bind(AI_RESOURCE_TARGET_TYPE)
    .bind(target_id)
    .bind(request_id)
    .bind(operator_id)
    .bind(operator_type)
    .bind(change_summary.to_string())
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to write AI resource audit log", error))?;
    Ok(())
}

fn row_error(error: sqlx::Error) -> DomainError {
    DomainError::new(error.to_string())
}

fn store_error(context: &str, error: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(database_error) = &error {
        let message = database_error.message().to_ascii_lowercase();
        if message.contains("unique") || message.contains("duplicate") {
            return DomainError::conflict(format!(
                "{context}: AI resource already exists ({})",
                database_error.message()
            ));
        }
    }
    DomainError::new(format!("{context}: {error}"))
}

#[cfg(test)]
#[path = "admin_ai_resource_store_tests.rs"]
mod tests;
