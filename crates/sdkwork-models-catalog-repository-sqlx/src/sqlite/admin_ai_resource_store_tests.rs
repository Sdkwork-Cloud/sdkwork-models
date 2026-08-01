use sqlx::sqlite::SqlitePoolOptions;

use super::*;
use sdkwork_models_contract_service::{AdminAiResourceGroupMemberCommand, AdminAiResourceSubject};

const TENANT_ID: i64 = 7;
const ORGANIZATION_ID: i64 = 9;
const GROUP_ID: i64 = 11;
const OPERATOR_ID: i64 = 13;
const REQUESTED_AT: &str = "2026-07-31T00:00:00Z";

#[tokio::test]
async fn member_hydration_reads_only_requested_page_resource_codes() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory SQLite");
    sqlx::query(
        r#"
        CREATE TABLE ai_resource_group_item (
            id INTEGER PRIMARY KEY,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL,
            status INTEGER NOT NULL,
            deleted_at TEXT,
            resource_group_code TEXT NOT NULL,
            resource_code TEXT NOT NULL,
            child_resource_group_code TEXT NOT NULL DEFAULT '',
            item_role TEXT,
            metadata TEXT,
            sort_order INTEGER
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create member table");
    sqlx::query(
        r#"
        INSERT INTO ai_resource_group_item
            (id, tenant_id, organization_id, status, resource_group_code, resource_code, item_role, metadata, sort_order)
        VALUES
            (1, 7, 9, 1, 'off-page', 'member.invalid', 'included', '{"required":"invalid"}', 1),
            (2, 7, 9, 1, 'page-resource', 'member.valid', 'fallback', '{"required":true}', 2)
        "#,
    )
    .execute(&pool)
    .await
    .expect("seed member rows");

    let members = load_members(
        &pool,
        TENANT_ID,
        ORGANIZATION_ID,
        &["page-resource".to_owned()],
    )
    .await
    .expect("off-page malformed members must not be hydrated");

    assert_eq!(members.len(), 1);
    let page_members = members
        .get("page-resource")
        .expect("requested page resource members");
    assert_eq!(page_members.len(), 1);
    assert_eq!(page_members[0].member_resource_code, "member.valid");
    assert!(page_members[0].required);
    assert!(!members.contains_key("off-page"));
}

#[tokio::test]
async fn member_mutations_commit_audit_and_routing_change_atomically() {
    let pool = member_test_pool().await;

    let first = upsert_ai_resource_group_member(
        &pool,
        upsert_command(
            "api.one",
            "included",
            Some(10),
            "member-1",
            "audit-1",
            "request-1",
        ),
    )
    .await
    .expect("insert first group member")
    .expect("group exists");
    assert_eq!(first.resource_code, "api.one");
    assert_eq!(first.member_role, "included");
    assert_eq!(first.sort_order, Some(10));

    let updated = upsert_ai_resource_group_member(
        &pool,
        upsert_command(
            "api.one",
            "fallback",
            Some(20),
            "member-2",
            "audit-2",
            "request-2",
        ),
    )
    .await
    .expect("update existing group member")
    .expect("group exists");
    assert_eq!(updated.member_role, "fallback");
    assert_eq!(updated.sort_order, Some(20));
    assert_eq!(active_member_count(&pool, "api.one").await, 1);
    assert_eq!(audit_count(&pool).await, 2);
    assert_eq!(routing_event_count(&pool).await, 2);
    assert_eq!(routing_version(&pool, TENANT_ID, ORGANIZATION_ID).await, 2);
    assert_eq!(routing_version(&pool, 0, 0).await, 2);

    let rollback_error = upsert_ai_resource_group_member(
        &pool,
        upsert_command(
            "api.two",
            "included",
            None,
            "member-3",
            "audit-2",
            "request-3",
        ),
    )
    .await
    .expect_err("duplicate audit UUID must roll back the member mutation");
    assert!(rollback_error
        .to_string()
        .contains("AI resource already exists"));
    assert_eq!(active_member_count(&pool, "api.two").await, 0);
    assert_eq!(audit_count(&pool).await, 2);
    assert_eq!(routing_event_count(&pool).await, 2);
    assert_eq!(routing_version(&pool, TENANT_ID, ORGANIZATION_ID).await, 2);

    let deleted =
        delete_ai_resource_group_member(&pool, delete_command("api.one", "audit-3", "request-4"))
            .await
            .expect("delete existing member");
    assert!(deleted);
    assert_eq!(active_member_count(&pool, "api.one").await, 0);
    assert_eq!(audit_count(&pool).await, 3);
    assert_eq!(routing_event_count(&pool).await, 3);
    assert_eq!(routing_version(&pool, TENANT_ID, ORGANIZATION_ID).await, 3);
    assert_eq!(routing_version(&pool, 0, 0).await, 3);

    let idempotent_delete =
        delete_ai_resource_group_member(&pool, delete_command("api.one", "audit-4", "request-5"))
            .await
            .expect("repeat member delete");
    assert!(idempotent_delete);
    assert_eq!(audit_count(&pool).await, 3);
    assert_eq!(routing_event_count(&pool).await, 3);
    assert_eq!(routing_version(&pool, TENANT_ID, ORGANIZATION_ID).await, 3);
}

#[test]
fn sqlite_member_mutations_use_write_lock_limit_and_transactional_events() {
    let source = include_str!("admin_ai_resource_store.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source before test module");
    let upsert = source
        .split("async fn upsert_ai_resource_group_member")
        .nth(1)
        .and_then(|body| {
            body.split("async fn delete_ai_resource_group_member")
                .next()
        })
        .expect("member upsert implementation");
    let delete = source
        .split("async fn delete_ai_resource_group_member")
        .nth(1)
        .and_then(|body| body.split("async fn delete_ai_resource_group").next())
        .expect("member delete implementation");

    assert!(upsert.contains("UPDATE ai_resource_group"));
    assert!(upsert.contains("member_count >= MAX_RESOURCE_GROUP_MEMBERS"));
    assert!(upsert.contains("dynamic API groups cannot maintain resource relationships"));
    assert!(upsert.contains("insert_audit_log("));
    assert!(upsert.contains("record_sqlite_ai_routing_config_change("));
    assert!(upsert.contains("tx.commit().await"));
    assert!(delete.contains("UPDATE ai_resource_group"));
    assert!(delete.contains("if result.rows_affected() > 0"));
    assert!(delete.contains("insert_audit_log("));
    assert!(delete.contains("record_sqlite_ai_routing_config_change("));
    assert!(delete.contains("Ok(true)"));
}

async fn member_test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory SQLite");
    for statement in MEMBER_TEST_SCHEMA {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("create member transaction test schema");
    }
    sqlx::query(
        r#"
        INSERT INTO ai_resource_group
            (id, tenant_id, organization_id, status, updated_at, group_code, group_name, group_type, selection_mode)
        VALUES (?, ?, ?, 1, ?, 'group.manual', 'Manual group', 'api_group', 'manual')
        "#,
    )
    .bind(GROUP_ID)
    .bind(TENANT_ID)
    .bind(ORGANIZATION_ID)
    .bind(REQUESTED_AT)
    .execute(&pool)
    .await
    .expect("seed resource group");
    for (id, resource_code) in [(21_i64, "api.one"), (22_i64, "api.two")] {
        sqlx::query(
            r#"
            INSERT INTO ai_resource
                (id, tenant_id, organization_id, status, resource_code, resource_type, display_name)
            VALUES (?, ?, ?, 1, ?, 'api_endpoint', ?)
            "#,
        )
        .bind(id)
        .bind(TENANT_ID)
        .bind(ORGANIZATION_ID)
        .bind(resource_code)
        .bind(resource_code)
        .execute(&pool)
        .await
        .expect("seed API resource");
    }
    pool
}

fn subject() -> AdminAiResourceSubject {
    AdminAiResourceSubject {
        tenant_id: TENANT_ID,
        organization_id: ORGANIZATION_ID,
        operator_id: OPERATOR_ID,
        operator_type: 1,
    }
}

fn upsert_command(
    resource_code: &str,
    item_role: &str,
    sort_order: Option<i64>,
    member_uuid: &str,
    audit_log_uuid: &str,
    request_id: &str,
) -> UpsertAdminAiResourceGroupMemberCommand {
    UpsertAdminAiResourceGroupMemberCommand {
        subject: subject(),
        group_id: GROUP_ID,
        member_uuid: member_uuid.to_owned(),
        audit_log_uuid: audit_log_uuid.to_owned(),
        member: AdminAiResourceGroupMemberCommand {
            resource_code: resource_code.to_owned(),
            item_role: item_role.to_owned(),
            sort_order,
        },
        request_id: request_id.to_owned(),
        requested_at: REQUESTED_AT.to_owned(),
    }
}

fn delete_command(
    resource_code: &str,
    audit_log_uuid: &str,
    request_id: &str,
) -> DeleteAdminAiResourceGroupMemberCommand {
    DeleteAdminAiResourceGroupMemberCommand {
        subject: subject(),
        group_id: GROUP_ID,
        resource_code: resource_code.to_owned(),
        audit_log_uuid: audit_log_uuid.to_owned(),
        request_id: request_id.to_owned(),
        requested_at: REQUESTED_AT.to_owned(),
    }
}

async fn active_member_count(pool: &SqlitePool, resource_code: &str) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM ai_resource_group_item
        WHERE resource_group_id = ?
          AND resource_code = ?
          AND status = 1
          AND deleted_at IS NULL
        "#,
    )
    .bind(GROUP_ID)
    .bind(resource_code)
    .fetch_one(pool)
    .await
    .expect("count active members")
}

async fn audit_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(1) FROM ops_audit_log")
        .fetch_one(pool)
        .await
        .expect("count audit rows")
}

async fn routing_event_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(1) FROM ai_config_change_event")
        .fetch_one(pool)
        .await
        .expect("count routing change events")
}

async fn routing_version(pool: &SqlitePool, tenant_id: i64, organization_id: i64) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT config_version
        FROM ai_config_version
        WHERE tenant_id = ? AND organization_id = ? AND config_scope = 'routing'
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .fetch_one(pool)
    .await
    .expect("load routing version")
}

const MEMBER_TEST_SCHEMA: &[&str] = &[
    r#"
    CREATE TABLE ai_resource_group (
        id INTEGER PRIMARY KEY,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        status INTEGER NOT NULL,
        updated_at TEXT NOT NULL,
        deleted_at TEXT,
        group_code TEXT NOT NULL,
        group_name TEXT NOT NULL,
        group_type TEXT,
        selection_mode TEXT,
        description TEXT,
        sort_order INTEGER
    )
    "#,
    r#"
    CREATE TABLE ai_resource (
        id INTEGER PRIMARY KEY,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        status INTEGER NOT NULL,
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
        sort_order INTEGER
    )
    "#,
    r#"
    CREATE TABLE ai_resource_group_item (
        id INTEGER PRIMARY KEY,
        uuid TEXT NOT NULL UNIQUE,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        data_scope INTEGER NOT NULL,
        status INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        version INTEGER NOT NULL DEFAULT 0,
        deleted_at TEXT,
        deleted_by INTEGER,
        metadata TEXT NOT NULL,
        resource_group_id INTEGER NOT NULL,
        resource_group_code TEXT NOT NULL,
        item_type TEXT NOT NULL,
        resource_id INTEGER,
        resource_code TEXT NOT NULL DEFAULT '',
        child_resource_group_id INTEGER,
        child_resource_group_code TEXT NOT NULL DEFAULT '',
        item_role TEXT,
        sort_order INTEGER,
        UNIQUE (tenant_id, organization_id, resource_group_id, item_type, resource_code, child_resource_group_code)
    )
    "#,
    r#"
    CREATE TABLE ops_audit_log (
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
    r#"
    CREATE TABLE ai_config_version (
        id INTEGER PRIMARY KEY,
        uuid TEXT NOT NULL UNIQUE,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        status INTEGER NOT NULL,
        version INTEGER NOT NULL DEFAULT 0,
        config_scope TEXT NOT NULL,
        config_version INTEGER NOT NULL,
        changed_object_type TEXT NOT NULL,
        changed_object_id INTEGER NOT NULL,
        published_at TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        UNIQUE (tenant_id, organization_id, config_scope)
    )
    "#,
    r#"
    CREATE TABLE ai_config_change_event (
        id INTEGER PRIMARY KEY,
        uuid TEXT NOT NULL UNIQUE,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        user_id INTEGER NOT NULL,
        request_id TEXT NOT NULL,
        payload_hash TEXT NOT NULL,
        status INTEGER NOT NULL,
        config_scope TEXT NOT NULL,
        changed_object_type TEXT NOT NULL,
        changed_object_id INTEGER NOT NULL,
        config_version INTEGER NOT NULL,
        event_status TEXT NOT NULL,
        event_payload TEXT NOT NULL,
        published_at TEXT NOT NULL,
        created_at TEXT NOT NULL
    )
    "#,
];
