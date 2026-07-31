use std::collections::HashMap;

use sdkwork_models_contract_service::{
    AdminAiResourceGroupItem, AdminAiResourceGroupListPage, AdminAiResourceGroupResourceItem,
    AdminAiResourceGroupResourcesPage, AdminAiResourceItem, AdminAiResourceListPage,
    AdminAiResourceMemberCommand, AdminAiResourceMemberItem, AdminAiResourceReadFuture,
    AdminAiResourceStore, CreateAdminAiResourceCommand, CreateAdminAiResourceGroupCommand,
    DeleteAdminAiResourceGroupCommand, DomainError, DomainResult,
    ListAdminAiResourceGroupResourcesQuery, ListAdminAiResourceGroupsQuery,
    ListAdminAiResourcesQuery, UpdateAdminAiResourceCommand, UpdateAdminAiResourceGroupCommand,
};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::routing_config_change::{
    record_postgres_ai_routing_config_change, AiRoutingConfigChange,
};
use crate::runtime_id::next_claw_runtime_id;

const AI_RESOURCE_TARGET_TYPE: i32 = 91;

struct ResolvedAiResourceGroupMember {
    item_type: &'static str,
    resource_id: Option<i64>,
    resource_code: String,
    child_resource_group_id: Option<i64>,
    child_resource_group_code: String,
}

#[derive(Debug, Clone)]
pub struct PostgresAdminAiResourceStore {
    pool: PgPool,
}

impl PostgresAdminAiResourceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AdminAiResourceStore for PostgresAdminAiResourceStore {
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
            record_postgres_ai_routing_config_change(
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
            record_postgres_ai_routing_config_change(
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

    fn delete_ai_resource_group<'a>(
        &'a self,
        command: DeleteAdminAiResourceGroupCommand,
    ) -> AdminAiResourceReadFuture<'a, bool> {
        Box::pin(async move { delete_ai_resource_group(&self.pool, command).await })
    }
}

async fn list_ai_resources(
    pool: &PgPool,
    query: ListAdminAiResourcesQuery,
) -> DomainResult<AdminAiResourceListPage> {
    let members =
        load_members(pool, query.subject.tenant_id, query.subject.organization_id).await?;
    let search = resource_search_pattern(query.q.as_deref());
    let total_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM ai_resource
        WHERE (
                (tenant_id = $1 AND organization_id = $2)
                OR (tenant_id = 0 AND organization_id = 0)
              )
          AND deleted_at IS NULL
          AND NOT (
              tenant_id = 0
              AND organization_id = 0
              AND ($1 <> 0 OR $2 <> 0)
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource tenant_resource
                  WHERE tenant_resource.tenant_id = $1
                    AND tenant_resource.organization_id = $2
                    AND tenant_resource.resource_code = ai_resource.resource_code
                    AND tenant_resource.deleted_at IS NULL
              )
          )
          AND (
              $3::text IS NULL
              OR resource_code ILIKE $3
              OR COALESCE(NULLIF(display_name, ''), resource_code) ILIKE $3
              OR COALESCE(resource_type, '') ILIKE $3
              OR COALESCE(vendor_code, '') ILIKE $3
              OR COALESCE(modality_code, '') ILIKE $3
              OR COALESCE(api_code, '') ILIKE $3
              OR COALESCE(catalog_key, '') ILIKE $3
              OR COALESCE(model, '') ILIKE $3
              OR COALESCE(provider_native_model, '') ILIKE $3
          )
        "#,
    )
    .bind(query.subject.tenant_id)
    .bind(query.subject.organization_id)
    .bind(search.as_deref())
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
            NULLIF(resource_schema ->> 'capability', '') AS capability,
            COALESCE(resource_schema -> 'capabilities', '[]'::jsonb)::text AS capabilities_json,
            COALESCE(
                NULLIF(resource_schema ->> 'compositionMode', ''),
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
                (tenant_id = $1 AND organization_id = $2)
                OR (tenant_id = 0 AND organization_id = 0)
              )
          AND deleted_at IS NULL
          AND NOT (
              tenant_id = 0
              AND organization_id = 0
              AND ($1 <> 0 OR $2 <> 0)
              AND EXISTS (
                  SELECT 1
                  FROM ai_resource tenant_resource
                  WHERE tenant_resource.tenant_id = $1
                    AND tenant_resource.organization_id = $2
                    AND tenant_resource.resource_code = ai_resource.resource_code
                    AND tenant_resource.deleted_at IS NULL
              )
          )
          AND (
              $3::text IS NULL
              OR resource_code ILIKE $3
              OR COALESCE(NULLIF(display_name, ''), resource_code) ILIKE $3
              OR COALESCE(resource_type, '') ILIKE $3
              OR COALESCE(vendor_code, '') ILIKE $3
              OR COALESCE(modality_code, '') ILIKE $3
              OR COALESCE(api_code, '') ILIKE $3
              OR COALESCE(catalog_key, '') ILIKE $3
              OR COALESCE(model, '') ILIKE $3
              OR COALESCE(provider_native_model, '') ILIKE $3
          )
        ORDER BY COALESCE(sort_order, 100000) ASC, id ASC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(query.subject.tenant_id)
    .bind(query.subject.organization_id)
    .bind(search.as_deref())
    .bind(query.normalized_limit())
    .bind(query.normalized_offset())
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to list AI resources", error))?;

    let items = rows
        .into_iter()
        .map(|row| item_from_row(row, &members))
        .collect::<DomainResult<Vec<_>>>()?;
    Ok(AdminAiResourceListPage { items, total_count })
}

async fn list_ai_resource_groups(
    pool: &PgPool,
    query: ListAdminAiResourceGroupsQuery,
) -> DomainResult<AdminAiResourceGroupListPage> {
    let search = resource_search_pattern(query.q.as_deref());
    let total_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM ai_resource_group g
        WHERE (
                (g.tenant_id = $1 AND g.organization_id = $2)
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
                  WHERE tenant_group.tenant_id = $1
                    AND tenant_group.organization_id = $2
                    AND tenant_group.group_code = g.group_code
                    AND tenant_group.deleted_at IS NULL
                    AND COALESCE(NULLIF(tenant_group.group_type, ''), 'api_group') = 'api_group'
              )
          )
          AND (
              $3::text IS NULL
              OR g.group_code ILIKE $3
              OR g.group_name ILIKE $3
              OR COALESCE(g.description, '') ILIKE $3
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
            (g.selection_mode = 'dynamic_all_api') AS dynamic
        FROM ai_resource_group g
        WHERE (
                (g.tenant_id = $1 AND g.organization_id = $2)
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
                  WHERE tenant_group.tenant_id = $1
                    AND tenant_group.organization_id = $2
                    AND tenant_group.group_code = g.group_code
                    AND tenant_group.deleted_at IS NULL
                    AND COALESCE(NULLIF(tenant_group.group_type, ''), 'api_group') = 'api_group'
              )
          )
          AND (
              $3::text IS NULL
              OR g.group_code ILIKE $3
              OR g.group_name ILIKE $3
              OR COALESCE(g.description, '') ILIKE $3
          )
        ORDER BY CASE WHEN g.tenant_id = $1 AND g.organization_id = $2 THEN 0 ELSE 1 END,
                 COALESCE(g.sort_order, 100000) ASC,
                 g.id ASC
        LIMIT $4 OFFSET $5
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
    pool: &PgPool,
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
    let dynamic = is_dynamic_group(group.group_code.as_str(), group.selection_mode.as_str());
    let search = resource_search_pattern(query.q.as_deref());
    let limit = query.normalized_limit();
    let offset = query.normalized_offset();

    let (total_count, rows) = if dynamic {
        let total_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM ai_resource r
            WHERE (
                    (r.tenant_id = $1 AND r.organization_id = $2)
                    OR (r.tenant_id = 0 AND r.organization_id = 0)
                  )
              AND r.resource_type = 'api_endpoint'
              AND r.deleted_at IS NULL
              AND NOT (
                  r.tenant_id = 0
                  AND r.organization_id = 0
                  AND ($1 <> 0 OR $2 <> 0)
                  AND EXISTS (
                      SELECT 1
                      FROM ai_resource tenant_resource
                      WHERE tenant_resource.tenant_id = $1
                        AND tenant_resource.organization_id = $2
                        AND tenant_resource.resource_code = r.resource_code
                        AND tenant_resource.deleted_at IS NULL
                  )
              )
              AND (
                  $3::text IS NULL
                  OR r.resource_code ILIKE $3
                  OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) ILIKE $3
                  OR COALESCE(r.resource_type, '') ILIKE $3
                  OR COALESCE(r.vendor_code, '') ILIKE $3
                  OR COALESCE(r.modality_code, '') ILIKE $3
                  OR COALESCE(r.api_code, '') ILIKE $3
                  OR COALESCE(r.catalog_key, '') ILIKE $3
                  OR COALESCE(r.model, '') ILIKE $3
                  OR COALESCE(r.provider_native_model, '') ILIKE $3
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
                    (r.tenant_id = $1 AND r.organization_id = $2)
                    OR (r.tenant_id = 0 AND r.organization_id = 0)
                  )
              AND r.resource_type = 'api_endpoint'
              AND r.deleted_at IS NULL
              AND NOT (
                  r.tenant_id = 0
                  AND r.organization_id = 0
                  AND ($1 <> 0 OR $2 <> 0)
                  AND EXISTS (
                      SELECT 1
                      FROM ai_resource tenant_resource
                      WHERE tenant_resource.tenant_id = $1
                        AND tenant_resource.organization_id = $2
                        AND tenant_resource.resource_code = r.resource_code
                        AND tenant_resource.deleted_at IS NULL
                  )
              )
              AND (
                  $3::text IS NULL
                  OR r.resource_code ILIKE $3
                  OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) ILIKE $3
                  OR COALESCE(r.resource_type, '') ILIKE $3
                  OR COALESCE(r.vendor_code, '') ILIKE $3
                  OR COALESCE(r.modality_code, '') ILIKE $3
                  OR COALESCE(r.api_code, '') ILIKE $3
                  OR COALESCE(r.catalog_key, '') ILIKE $3
                  OR COALESCE(r.model, '') ILIKE $3
                  OR COALESCE(r.provider_native_model, '') ILIKE $3
              )
            ORDER BY COALESCE(r.sort_order, 100000) ASC, r.id ASC
            LIMIT $4 OFFSET $5
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
            WHERE item.tenant_id = $1
              AND item.organization_id = $2
              AND item.resource_group_id = $3
              AND item.item_type = 'resource'
              AND item.deleted_at IS NULL
              AND item.status = 1
              AND (
                  $4::text IS NULL
                  OR r.resource_code ILIKE $4
                  OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) ILIKE $4
                  OR COALESCE(r.resource_type, '') ILIKE $4
                  OR COALESCE(r.vendor_code, '') ILIKE $4
                  OR COALESCE(r.modality_code, '') ILIKE $4
                  OR COALESCE(r.api_code, '') ILIKE $4
                  OR COALESCE(r.catalog_key, '') ILIKE $4
                  OR COALESCE(r.model, '') ILIKE $4
                  OR COALESCE(r.provider_native_model, '') ILIKE $4
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
            WHERE item.tenant_id = $1
              AND item.organization_id = $2
              AND item.resource_group_id = $3
              AND item.item_type = 'resource'
              AND item.deleted_at IS NULL
              AND item.status = 1
              AND (
                  $4::text IS NULL
                  OR r.resource_code ILIKE $4
                  OR COALESCE(NULLIF(r.display_name, ''), r.resource_code) ILIKE $4
                  OR COALESCE(r.resource_type, '') ILIKE $4
                  OR COALESCE(r.vendor_code, '') ILIKE $4
                  OR COALESCE(r.modality_code, '') ILIKE $4
                  OR COALESCE(r.api_code, '') ILIKE $4
                  OR COALESCE(r.catalog_key, '') ILIKE $4
                  OR COALESCE(r.model, '') ILIKE $4
                  OR COALESCE(r.provider_native_model, '') ILIKE $4
              )
            ORDER BY COALESCE(item.sort_order, r.sort_order, 100000) ASC, item.id ASC
            LIMIT $5 OFFSET $6
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
    pool: &PgPool,
    command: CreateAdminAiResourceGroupCommand,
) -> DomainResult<AdminAiResourceGroupItem> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| store_error("failed to begin AI resource group transaction", error))?;
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
            "selectionMode": &command.selection_mode,
            "memberCount": command.members.len()
        }),
    )
    .await?;
    record_postgres_ai_routing_config_change(
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
                "selectionMode": &command.selection_mode,
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
    tx.commit()
        .await
        .map_err(|error| store_error("failed to commit AI resource group transaction", error))?;
    Ok(item)
}

async fn update_ai_resource_group(
    pool: &PgPool,
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
    let effective_group_code = command
        .group_code
        .as_deref()
        .unwrap_or(current.group_code.as_str());
    let effective_selection_mode = command
        .selection_mode
        .as_deref()
        .unwrap_or(current.selection_mode.as_str());
    let effective_dynamic = is_dynamic_group(effective_group_code, effective_selection_mode);
    if effective_dynamic
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
    if command.group_code.is_some() && effective_group_code != current.group_code.as_str() {
        rename_group_member_parent_code(
            &mut tx,
            command.group_id,
            &current.group_code,
            effective_group_code,
            &command,
        )
        .await?;
    }
    if effective_dynamic {
        replace_group_members_for_update(&mut tx, effective_group_code, &[], &command).await?;
    } else if let Some(members) = command.members.as_ref() {
        replace_group_members_for_update(&mut tx, effective_group_code, members, &command).await?;
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
            "selectionModeChanged": command.selection_mode.is_some(),
            "membersChanged": command.members.is_some()
        }),
    )
    .await?;
    record_postgres_ai_routing_config_change(
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
                "selectionModeChanged": command.selection_mode.is_some(),
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
    .await?
    .ok_or_else(|| DomainError::new("updated AI resource group could not be reloaded"))?;
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource group update transaction",
            error,
        )
    })?;
    Ok(Some(item))
}

async fn delete_ai_resource_group(
    pool: &PgPool,
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
            deleted_at = $1,
            deleted_by = $2,
            updated_at = $3,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $4
          AND organization_id = $5
          AND resource_group_id = $6
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
    .map_err(|error| store_error("failed to delete AI resource group members", error))?;
    let affected = sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET status = -1,
            deleted_at = $1,
            deleted_by = $2,
            updated_at = $3,
            version = COALESCE(version, 0) + 1
        WHERE id = $4
          AND tenant_id = $5
          AND organization_id = $6
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
    .map_err(|error| store_error("failed to delete AI resource group", error))?
    .rows_affected();
    if affected == 0 {
        return Ok(false);
    }
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
            "groupId": command.group_id,
            "groupCode": current.group_code
        }),
    )
    .await?;
    record_postgres_ai_routing_config_change(
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
                "groupId": command.group_id,
                "groupCode": current.group_code
            }),
        ),
    )
    .await?;
    tx.commit().await.map_err(|error| {
        store_error(
            "failed to commit AI resource group delete transaction",
            error,
        )
    })?;
    Ok(true)
}

async fn insert_ai_resource(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateAdminAiResourceCommand,
) -> DomainResult<i64> {
    sqlx::query_scalar(
        r#"
        INSERT INTO ai_resource
            (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, resource_code, resource_type, display_name, vendor_code, modality_code, api_code, catalog_key, model, provider_native_model, resource_schema, sort_order, id)
        VALUES
            ($1, $2, $3, 1, $4, $5, $6, 0, '{}'::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16::jsonb, $17, $18)
        RETURNING id
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
    .bind(serde_json::json!({ "compositionMode": &command.composition_mode }).to_string())
    .bind(command.sort_order)
    .bind(next_claw_runtime_id("ai_resource")?)
    .fetch_one(&mut **tx)
    .await
        .map_err(|error| store_error("failed to create AI resource", error))
}

async fn insert_ai_resource_group(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateAdminAiResourceGroupCommand,
) -> DomainResult<i64> {
    sqlx::query_scalar(
        r#"
        INSERT INTO ai_resource_group
            (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, group_code, group_name, group_type, selection_mode, description, sort_order, id)
        VALUES
            ($1, $2, $3, 1, $4, $5, $6, 0, '{}'::jsonb, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id
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
    .bind(next_claw_runtime_id("ai_resource_group")?)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create AI resource group", error))
}

async fn update_ai_resource_core(
    tx: &mut Transaction<'_, Postgres>,
    command: &UpdateAdminAiResourceCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource
        SET resource_code = COALESCE($1, resource_code),
            resource_type = COALESCE($2, resource_type),
            display_name = COALESCE($3, display_name),
            vendor_code = CASE WHEN $4 THEN $5 ELSE vendor_code END,
            modality_code = CASE WHEN $6 THEN $7 ELSE modality_code END,
            api_code = CASE WHEN $8 THEN $9 ELSE api_code END,
            catalog_key = CASE WHEN $10 THEN $11 ELSE catalog_key END,
            model = CASE WHEN $12 THEN $13 ELSE model END,
            provider_native_model = CASE WHEN $14 THEN $15 ELSE provider_native_model END,
            resource_schema = CASE
                WHEN $16 IS NULL THEN resource_schema
                ELSE jsonb_set(COALESCE(resource_schema, '{}'::jsonb), '{compositionMode}', to_jsonb($16::text), true)
            END,
            status = COALESCE($17, status),
            sort_order = CASE WHEN $18 THEN $19 ELSE sort_order END,
            updated_at = $20,
            version = COALESCE(version, 0) + 1
        WHERE id = $21
          AND tenant_id = $22
          AND organization_id = $23
          AND deleted_at IS NULL
        "#,
    )
    .bind(command.resource_code.as_deref())
    .bind(command.resource_type.as_deref())
    .bind(command.display_name.as_deref())
    .bind(command.vendor_code.is_some())
    .bind(optional_optional_str(&command.vendor_code))
    .bind(command.modality_code.is_some())
    .bind(optional_optional_str(&command.modality_code))
    .bind(command.api_endpoint_code.is_some())
    .bind(optional_optional_str(&command.api_endpoint_code))
    .bind(command.catalog_key.is_some())
    .bind(optional_optional_str(&command.catalog_key))
    .bind(command.model.is_some())
    .bind(optional_optional_str(&command.model))
    .bind(command.provider_native_model.is_some())
    .bind(optional_optional_str(&command.provider_native_model))
    .bind(command.composition_mode.as_deref())
    .bind(command.status.as_ref().map(|value| status_code(value)))
    .bind(command.sort_order.is_some())
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
    tx: &mut Transaction<'_, Postgres>,
    command: &UpdateAdminAiResourceGroupCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET group_code = COALESCE($1, group_code),
            group_name = COALESCE($2, group_name),
            group_type = COALESCE($3, group_type),
            selection_mode = COALESCE($4, selection_mode),
            description = CASE WHEN $5 THEN $6 ELSE description END,
            sort_order = CASE WHEN $7 THEN $8 ELSE sort_order END,
            status = COALESCE($9, status),
            updated_at = $10,
            version = COALESCE(version, 0) + 1
        WHERE id = $11
          AND tenant_id = $12
          AND organization_id = $13
          AND deleted_at IS NULL
        "#,
    )
    .bind(command.group_code.as_deref())
    .bind(command.group_name.as_deref())
    .bind(command.group_type.as_deref())
    .bind(command.selection_mode.as_deref())
    .bind(command.description.is_some())
    .bind(optional_optional_str(&command.description))
    .bind(command.sort_order.is_some())
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
    tx: &mut Transaction<'_, Postgres>,
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
    tx: &mut Transaction<'_, Postgres>,
    previous_parent_resource_code: &str,
    effective_parent_resource_code: &str,
    members: &[AdminAiResourceMemberCommand],
    command: &UpdateAdminAiResourceCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = -1, deleted_at = $1, deleted_by = $2, updated_at = $3,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $4
          AND organization_id = $5
          AND resource_group_code = $6
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

async fn replace_group_members_for_create(
    tx: &mut Transaction<'_, Postgres>,
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
    tx: &mut Transaction<'_, Postgres>,
    effective_group_code: &str,
    members: &[sdkwork_models_contract_service::AdminAiResourceGroupMemberCommand],
    command: &UpdateAdminAiResourceGroupCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET status = -1, deleted_at = $1, deleted_by = $2, updated_at = $3,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $4
          AND organization_id = $5
          AND resource_group_id = $6
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

async fn rename_group_member_parent_code(
    tx: &mut Transaction<'_, Postgres>,
    group_id: i64,
    previous_group_code: &str,
    effective_group_code: &str,
    command: &UpdateAdminAiResourceGroupCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET resource_group_code = $1,
            updated_at = $2,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $3
          AND organization_id = $4
          AND resource_group_id = $5
          AND resource_group_code = $6
          AND deleted_at IS NULL
        "#,
    )
    .bind(effective_group_code)
    .bind(&command.requested_at)
    .bind(command.subject.tenant_id)
    .bind(command.subject.organization_id)
    .bind(group_id)
    .bind(previous_group_code)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to rename AI resource group members", error))?;
    Ok(())
}

async fn rename_members_for_resource_code(
    tx: &mut Transaction<'_, Postgres>,
    _parent_resource_id: i64,
    previous_parent_resource_code: &str,
    effective_parent_resource_code: &str,
    command: &UpdateAdminAiResourceCommand,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group_item
        SET resource_group_code = $1, updated_at = $2,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $3
          AND organization_id = $4
          AND resource_group_code = $5
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
        SET group_code = $1, updated_at = $2,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $3
          AND organization_id = $4
          AND group_code = $5
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

async fn insert_group_resource_members(
    tx: &mut Transaction<'_, Postgres>,
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
                    (tenant_id = $1 AND organization_id = $2)
                    OR (tenant_id = 0 AND organization_id = 0)
                  )
              AND resource_code = $3
              AND resource_type = 'api_endpoint'
              AND deleted_at IS NULL
            ORDER BY CASE WHEN tenant_id = $1 AND organization_id = $2 THEN 0 ELSE 1 END,
                     id ASC
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&member.resource_code)
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
                ($1, $2, $3, 1, 1, $4, $5, 0, '{}'::jsonb, $6, $7, 'resource', $8, $9, NULL, '', $10, $11, $12)
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

async fn sync_resource_group_status(
    tx: &mut Transaction<'_, Postgres>,
    resource_code: &str,
    command: &UpdateAdminAiResourceCommand,
    status: i32,
) -> DomainResult<()> {
    sqlx::query(
        r#"
        UPDATE ai_resource_group
        SET status = $1,
            updated_at = $2,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $3
          AND organization_id = $4
          AND group_code = $5
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
        SET status = $1,
            updated_at = $2,
            version = COALESCE(version, 0) + 1
        WHERE tenant_id = $3
          AND organization_id = $4
          AND resource_group_code = $5
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
    tx: &mut Transaction<'_, Postgres>,
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
                ($1, $2, $3, 1, 1, $4, $5, 0, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    organization_id: i64,
    member_resource_code: &str,
) -> DomainResult<ResolvedAiResourceGroupMember> {
    if let Some(resource_group_id) = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM ai_resource_group
        WHERE tenant_id = $1
          AND organization_id = $2
          AND group_code = $3
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
        WHERE tenant_id = $1
          AND organization_id = $2
          AND resource_code = $3
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
    tx: &mut Transaction<'_, Postgres>,
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
        WHERE tenant_id = $1
          AND organization_id = $2
          AND group_code = $3
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

    if let Some(group_id) = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE ai_resource_group
        SET group_code = $1,
            group_name = COALESCE((
                SELECT NULLIF(display_name, '')
                FROM ai_resource r
                WHERE r.id = $2
                  AND r.tenant_id = ai_resource_group.tenant_id
                  AND r.organization_id = ai_resource_group.organization_id
                  AND r.deleted_at IS NULL
                LIMIT 1
            ), $1),
            selection_mode = COALESCE(NULLIF(selection_mode, ''), $3),
            status = 1,
            deleted_at = NULL,
            deleted_by = NULL,
            updated_at = $4,
            version = COALESCE(version, 0) + 1
        WHERE uuid = $5
          AND tenant_id = $6
          AND organization_id = $7
        RETURNING id
        "#,
    )
    .bind(resource_code)
    .bind(resource_id)
    .bind(composition_mode)
    .bind(requested_at)
    .bind(&group_uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to update resource group", error))?
    {
        return Ok(group_id);
    }

    sqlx::query_scalar(
        r#"
        INSERT INTO ai_resource_group
            (uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at, version, metadata, group_code, group_name, group_type, selection_mode, sort_order, id)
        SELECT
            $7,
            tenant_id,
            organization_id,
            data_scope,
            status,
            $1,
            $2,
            0,
            '{}'::jsonb,
            resource_code,
            COALESCE(NULLIF(display_name, ''), resource_code),
            resource_type,
            $3,
            sort_order,
            $8
        FROM ai_resource
        WHERE id = $4
          AND tenant_id = $5
          AND organization_id = $6
          AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(requested_at)
    .bind(requested_at)
    .bind(composition_mode)
    .bind(resource_id)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(group_uuid)
    .bind(next_claw_runtime_id("ai_resource_group")?)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create resource group", error))
}

async fn load_resource_by_id(
    tx: &mut Transaction<'_, Postgres>,
    resource_id: i64,
    tenant_id: i64,
    organization_id: i64,
) -> DomainResult<Option<AdminAiResourceItem>> {
    let members = load_members_tx(tx, tenant_id, organization_id).await?;
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
            NULLIF(resource_schema ->> 'capability', '') AS capability,
            COALESCE(resource_schema -> 'capabilities', '[]'::jsonb)::text AS capabilities_json,
            COALESCE(
                NULLIF(resource_schema ->> 'compositionMode', ''),
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
        WHERE id = $1
          AND tenant_id = $2
          AND organization_id = $3
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

    row.map(|row| item_from_row(row, &members)).transpose()
}

struct ResourceGroupHeader {
    id: i64,
    tenant_id: i64,
    organization_id: i64,
    group_code: String,
    selection_mode: String,
}

async fn load_group_header(
    pool: &PgPool,
    tenant_id: i64,
    organization_id: i64,
    group_id_or_code: &str,
) -> DomainResult<Option<ResourceGroupHeader>> {
    let trimmed = group_id_or_code.trim();
    let row = if let Ok(group_id) = trimmed.parse::<i64>() {
        sqlx::query(
            r#"
            SELECT
                id,
                tenant_id,
                organization_id,
                group_code,
                COALESCE(NULLIF(selection_mode, ''), 'manual') AS selection_mode
            FROM ai_resource_group
            WHERE id = $1
              AND (
                    (tenant_id = $2 AND organization_id = $3)
                    OR (tenant_id = 0 AND organization_id = 0)
                  )
              AND deleted_at IS NULL
              AND COALESCE(NULLIF(group_type, ''), 'api_group') = 'api_group'
            ORDER BY CASE WHEN tenant_id = $2 AND organization_id = $3 THEN 0 ELSE 1 END,
                     id ASC
            LIMIT 1
            "#,
        )
        .bind(group_id)
        .bind(tenant_id)
        .bind(organization_id)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT
                id,
                tenant_id,
                organization_id,
                group_code,
                COALESCE(NULLIF(selection_mode, ''), 'manual') AS selection_mode
            FROM ai_resource_group
            WHERE group_code = $1
              AND (
                    (tenant_id = $2 AND organization_id = $3)
                    OR (tenant_id = 0 AND organization_id = 0)
                  )
              AND deleted_at IS NULL
              AND COALESCE(NULLIF(group_type, ''), 'api_group') = 'api_group'
            ORDER BY CASE WHEN tenant_id = $2 AND organization_id = $3 THEN 0 ELSE 1 END,
                     id ASC
            LIMIT 1
            "#,
        )
        .bind(trimmed)
        .bind(tenant_id)
        .bind(organization_id)
        .fetch_optional(pool)
        .await
    }
    .map_err(|error| store_error("failed to load AI resource group", error))?;

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
    tx: &mut Transaction<'_, Postgres>,
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
            (g.selection_mode = 'dynamic_all_api') AS dynamic
        FROM ai_resource_group g
        WHERE g.id = $1
          AND g.tenant_id = $2
          AND g.organization_id = $3
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

async fn load_members(
    pool: &PgPool,
    tenant_id: i64,
    organization_id: i64,
) -> DomainResult<HashMap<String, Vec<AdminAiResourceMemberItem>>> {
    let rows = sqlx::query(
        r#"
        SELECT
            resource_group_code AS parent_resource_code,
            COALESCE(NULLIF(resource_code, ''), child_resource_group_code) AS member_resource_code,
            COALESCE(NULLIF(item_role, ''), 'included') AS member_role,
            COALESCE((metadata ->> 'required')::boolean, true) AS required,
            sort_order
        FROM ai_resource_group_item
        WHERE tenant_id = $1
          AND organization_id = $2
          AND deleted_at IS NULL
          AND status = 1
        ORDER BY resource_group_code ASC, COALESCE(sort_order, 100000) ASC, id ASC
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to list AI resource members", error))?;

    members_from_rows(rows)
}

async fn load_members_tx(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    organization_id: i64,
) -> DomainResult<HashMap<String, Vec<AdminAiResourceMemberItem>>> {
    let rows = sqlx::query(
        r#"
        SELECT
            resource_group_code AS parent_resource_code,
            COALESCE(NULLIF(resource_code, ''), child_resource_group_code) AS member_resource_code,
            COALESCE(NULLIF(item_role, ''), 'included') AS member_role,
            COALESCE((metadata ->> 'required')::boolean, true) AS required,
            sort_order
        FROM ai_resource_group_item
        WHERE tenant_id = $1
          AND organization_id = $2
          AND deleted_at IS NULL
          AND status = 1
        ORDER BY resource_group_code ASC, COALESCE(sort_order, 100000) ASC, id ASC
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|error| store_error("failed to list AI resource members", error))?;

    members_from_rows(rows)
}

fn members_from_rows(
    rows: Vec<sqlx::postgres::PgRow>,
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
                required: row.try_get("required").map_err(row_error)?,
                sort_order: optional_int4_as_i64_cell(&row, "sort_order")?,
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
    row: sqlx::postgres::PgRow,
    members: &HashMap<String, Vec<AdminAiResourceMemberItem>>,
) -> DomainResult<AdminAiResourceItem> {
    let resource_code: String = row.try_get("resource_code").map_err(row_error)?;
    let status: i32 = row.try_get("status").map_err(row_error)?;
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
        capability: optional_string_cell(&row, "capability"),
        capabilities: string_array_cell(&row, "capabilities_json")?,
        composition_mode: row.try_get("composition_mode").map_err(row_error)?,
        status: status_label(status),
        sort_order: optional_int4_as_i64_cell(&row, "sort_order")?,
        members: members.get(&resource_code).cloned().unwrap_or_default(),
    })
}

fn group_item_from_row(row: sqlx::postgres::PgRow) -> DomainResult<AdminAiResourceGroupItem> {
    let status: i32 = row.try_get("status").map_err(row_error)?;
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
        sort_order: optional_int4_as_i64_cell(&row, "sort_order")?,
        status: status_label(status),
        resource_count: row.try_get("resource_count").map_err(row_error)?,
        dynamic: row.try_get("dynamic").map_err(row_error)?,
    })
}

#[derive(Default)]
struct AiResourceGroupSummary {
    vendor_codes: Vec<String>,
    capabilities: Vec<String>,
}

async fn hydrate_group_summaries(
    pool: &PgPool,
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
                NULLIF(r.resource_schema ->> 'capability', ''),
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
                (g.tenant_id = $1 AND g.organization_id = $2)
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
                  WHERE tenant_group.tenant_id = $1
                    AND tenant_group.organization_id = $2
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
    row: sqlx::postgres::PgRow,
) -> DomainResult<AdminAiResourceGroupResourceItem> {
    let status: i32 = row.try_get("status").map_err(row_error)?;
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
        sort_order: optional_int4_as_i64_cell(&row, "sort_order")?,
        member_role: row.try_get("member_role").map_err(row_error)?,
    })
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, name: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(name)
        .ok()
        .flatten()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn optional_int4_as_i64_cell(row: &sqlx::postgres::PgRow, name: &str) -> DomainResult<Option<i64>> {
    row.try_get::<Option<i32>, _>(name)
        .map(|value| value.map(i64::from))
        .map_err(row_error)
}

fn string_array_cell(row: &sqlx::postgres::PgRow, name: &str) -> DomainResult<Vec<String>> {
    let raw = row
        .try_get::<Option<String>, _>(name)
        .map_err(row_error)?
        .unwrap_or_else(|| "[]".to_owned());
    parse_string_array_json(&raw, name)
}

fn string_array_cell_or_empty(
    row: &sqlx::postgres::PgRow,
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

fn status_label(status: i32) -> String {
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
    tx: &mut Transaction<'_, Postgres>,
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
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::jsonb)
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
