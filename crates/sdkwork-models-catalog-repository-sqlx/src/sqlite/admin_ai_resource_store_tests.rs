use sqlx::sqlite::SqlitePoolOptions;

use super::*;
use sdkwork_models_contract_service::{
    AdminAiResourceGroupMemberCommand, AdminAiResourceHierarchyNodeCommand,
    AdminAiResourceMemberCommand, AdminAiResourceSubject, CreateAdminAiResourceCommand,
    ListAdminAiResourcesQuery, ReplaceAdminAiResourceHierarchyCommand,
    UpdateAdminAiResourceCommand,
};

const TENANT_ID: i64 = 7;
const ORGANIZATION_ID: i64 = 9;
const GROUP_ID: i64 = 11;
const OPERATOR_ID: i64 = 13;
const REQUESTED_AT: &str = "2026-07-31T00:00:00Z";

#[tokio::test]
async fn model_access_channel_metadata_round_trips_without_credentials() {
    let pool = access_channel_test_pool().await;
    let create = CreateAdminAiResourceCommand {
        subject: subject(),
        resource_uuid: "channel-uuid".to_owned(),
        member_uuids: Vec::new(),
        audit_log_uuid: "unused-audit".to_owned(),
        resource_code: "channel.relay".to_owned(),
        resource_type: "model_access_channel".to_owned(),
        display_name: "Relay".to_owned(),
        vendor_code: None,
        modality_code: None,
        api_endpoint_code: None,
        catalog_key: None,
        model: None,
        provider_native_model: None,
        access_channel_kind: Some("relay".to_owned()),
        base_url: Some("https://relay.example.test/v1".to_owned()),
        default_vendor_code: Some("openai".to_owned()),
        default_model_id: Some("gpt-5".to_owned()),
        supported_agent_provider_ids: vec!["codex".to_owned(), "gemini".to_owned()],
        description: Some("Shared relay".to_owned()),
        composition_mode: "all".to_owned(),
        status: "active".to_owned(),
        sort_order: Some(10),
        members: Vec::new(),
        request_id: "unused-request".to_owned(),
        requested_at: REQUESTED_AT.to_owned(),
    };
    let schema = ai_resource_schema_for_create(&create);
    assert!(!schema.to_ascii_lowercase().contains("apikey"));

    let mut tx = pool.begin().await.expect("begin create transaction");
    let channel_id = insert_ai_resource(&mut tx, &create)
        .await
        .expect("insert access channel");
    tx.commit().await.expect("commit access channel");
    seed_channel_model_member(&pool).await;

    let page = list_ai_resources(
        &pool,
        ListAdminAiResourcesQuery {
            subject: subject(),
            q: Some("gpt-5".to_owned()),
            resource_type: Some("model_access_channel".to_owned()),
            status: Some("active".to_owned()),
            access_channel_kind: Some("relay".to_owned()),
            vendor_code: Some("openai".to_owned()),
            agent_provider_id: Some("codex".to_owned()),
            require_valid_access_channel_metadata: true,
            limit: Some(20),
            offset: Some(0),
        },
    )
    .await
    .expect("list filtered access channels");
    assert_eq!(page.total_count, 1);
    let channel = page.items.first().expect("access channel item");
    assert_eq!(channel.access_channel_kind.as_deref(), Some("relay"));
    assert_eq!(
        channel.base_url.as_deref(),
        Some("https://relay.example.test/v1")
    );
    assert_eq!(
        channel.supported_agent_provider_ids,
        vec!["codex".to_owned(), "gemini".to_owned()]
    );
    assert_eq!(channel.description.as_deref(), Some("Shared relay"));
    assert_eq!(channel.default_vendor_code.as_deref(), Some("openai"));
    assert_eq!(channel.default_model_id.as_deref(), Some("gpt-5"));
    assert_eq!(channel.members.len(), 1);

    let update = UpdateAdminAiResourceCommand {
        subject: subject(),
        resource_id: channel_id,
        member_uuids: Vec::new(),
        audit_log_uuid: "unused-update-audit".to_owned(),
        resource_code: None,
        resource_type: None,
        display_name: None,
        vendor_code: None,
        modality_code: None,
        api_endpoint_code: None,
        catalog_key: None,
        model: None,
        provider_native_model: None,
        access_channel_kind: Some("official".to_owned()),
        base_url: Some("https://api.openai.com/v1".to_owned()),
        default_vendor_code: Some("openai".to_owned()),
        default_model_id: Some("gpt-5.1".to_owned()),
        supported_agent_provider_ids: Some(vec!["codex".to_owned()]),
        description: Some(None),
        composition_mode: None,
        status: None,
        sort_order: None,
        members: None,
        request_id: "unused-update-request".to_owned(),
        requested_at: REQUESTED_AT.to_owned(),
    };
    let mut tx = pool.begin().await.expect("begin update transaction");
    update_ai_resource_core(&mut tx, &update)
        .await
        .expect("update access channel metadata");
    tx.commit().await.expect("commit access channel update");
    let resource_schema: String =
        sqlx::query_scalar("SELECT resource_schema FROM ai_resource WHERE id = ?")
            .bind(channel_id)
            .fetch_one(&pool)
            .await
            .expect("load updated resource schema");
    let resource_schema: serde_json::Value =
        serde_json::from_str(&resource_schema).expect("valid resource schema JSON");
    assert_eq!(resource_schema["accessChannelKind"], "official");
    assert_eq!(resource_schema["baseUrl"], "https://api.openai.com/v1");
    assert_eq!(resource_schema["defaultVendorCode"], "openai");
    assert_eq!(resource_schema["defaultModelId"], "gpt-5.1");
    assert_eq!(
        resource_schema["supportedAgentProviderIds"],
        serde_json::json!(["codex"])
    );
    assert!(resource_schema.get("description").is_none());
    assert!(resource_schema.get("apiKey").is_none());
}

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

#[tokio::test]
async fn hierarchy_replacement_is_atomic_and_retires_removed_descendants() {
    let pool = hierarchy_test_pool().await;
    let first = replace_ai_resource_hierarchy(
        &pool,
        hierarchy_command(&["gpt-5", "gpt-4.1"], "audit-h1", "request-h1"),
    )
    .await
    .expect("create hierarchy");
    assert_eq!(first.resource_code, "channel.relay");
    assert_eq!(first.members.len(), 1);
    assert_eq!(active_hierarchy_resource_count(&pool).await, 4);
    let stale_resource_id: i64 = sqlx::query_scalar(
        "SELECT id FROM ai_resource WHERE resource_code = 'channel.relay.model.1.2'",
    )
    .fetch_one(&pool)
    .await
    .expect("load second model id");

    replace_ai_resource_hierarchy(
        &pool,
        hierarchy_command(&["gpt-5"], "audit-h2", "request-h2"),
    )
    .await
    .expect("shrink hierarchy");
    assert_eq!(active_hierarchy_resource_count(&pool).await, 3);
    let stale_state: (i64, Option<String>) =
        sqlx::query_as("SELECT status, deleted_at FROM ai_resource WHERE id = ?")
            .bind(stale_resource_id)
            .fetch_one(&pool)
            .await
            .expect("load retired model");
    assert_eq!(stale_state.0, -1);
    assert!(stale_state.1.is_some());
    let stale_member_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM ai_resource_group_item
        WHERE (resource_group_code = 'channel.relay.model.1.2'
               OR resource_code = 'channel.relay.model.1.2')
          AND status = 1
          AND deleted_at IS NULL
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("count stale member references");
    assert_eq!(stale_member_count, 0);

    let mut rollback = hierarchy_command(&["gpt-5"], "audit-h2", "request-h-rollback");
    rollback.nodes.last_mut().expect("root node").display_name = "Must Roll Back".to_owned();
    replace_ai_resource_hierarchy(&pool, rollback)
        .await
        .expect_err("duplicate audit UUID must roll back the whole graph");
    let root_name: String = sqlx::query_scalar(
        "SELECT display_name FROM ai_resource WHERE resource_code = 'channel.relay'",
    )
    .fetch_one(&pool)
    .await
    .expect("load root after rollback");
    assert_eq!(root_name, "Relay");
    assert_eq!(audit_count(&pool).await, 2);
    assert_eq!(routing_event_count(&pool).await, 2);

    replace_ai_resource_hierarchy(
        &pool,
        hierarchy_command(&["gpt-5", "gpt-4.1"], "audit-h3", "request-h3"),
    )
    .await
    .expect("revive retired descendant");
    let revived_state: (i64, i64, Option<String>) = sqlx::query_as(
        "SELECT id, status, deleted_at FROM ai_resource WHERE resource_code = 'channel.relay.model.1.2'",
    )
    .fetch_one(&pool)
    .await
    .expect("load revived model");
    assert_eq!(revived_state.0, stale_resource_id);
    assert_eq!(revived_state.1, 1);
    assert!(revived_state.2.is_none());
    assert_eq!(active_hierarchy_resource_count(&pool).await, 4);
    assert_eq!(audit_count(&pool).await, 3);
    assert_eq!(routing_event_count(&pool).await, 3);
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

async fn access_channel_test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect access channel SQLite");
    for &statement in ACCESS_CHANNEL_TEST_SCHEMA {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("create access channel test schema");
    }
    pool
}

async fn seed_channel_model_member(pool: &SqlitePool) {
    sqlx::query(
        r#"
        INSERT INTO ai_resource
            (id, uuid, tenant_id, organization_id, data_scope, status, created_at, updated_at,
             version, metadata, resource_code, resource_type, display_name, vendor_code,
             catalog_key, model, provider_native_model, resource_schema)
        VALUES
            (200, 'model-resource-uuid', 7, 9, 1, 1, ?, ?, 0, '{}', 'model.openai.gpt-5',
             'model_api', 'GPT-5', 'openai', 'openai/gpt-5', 'gpt-5', 'gpt-5', '{}')
        "#,
    )
    .bind(REQUESTED_AT)
    .bind(REQUESTED_AT)
    .execute(pool)
    .await
    .expect("seed model resource");
    sqlx::query(
        r#"
        INSERT INTO ai_resource_group_item
            (id, tenant_id, organization_id, status, resource_group_code, resource_code,
             child_resource_group_code, item_role, metadata, sort_order)
        VALUES
            (300, 7, 9, 1, 'channel.relay', 'model.openai.gpt-5', '', 'included',
             '{"required":true}', 1)
        "#,
    )
    .execute(pool)
    .await
    .expect("seed access channel member");
}

const ACCESS_CHANNEL_TEST_SCHEMA: &[&str] = &[
    r#"
    CREATE TABLE ai_resource (
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
    CREATE TABLE ai_resource_group (
        id INTEGER PRIMARY KEY,
        tenant_id INTEGER NOT NULL,
        organization_id INTEGER NOT NULL,
        group_code TEXT NOT NULL,
        selection_mode TEXT,
        deleted_at TEXT
    )
    "#,
    r#"
    CREATE TABLE ai_resource_group_item (
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
];

async fn member_test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory SQLite");
    for &statement in MEMBER_TEST_SCHEMA {
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

async fn hierarchy_test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect hierarchy SQLite");
    for &statement in HIERARCHY_TEST_SCHEMA {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("create hierarchy test schema");
    }
    pool
}

fn hierarchy_command(
    model_ids: &[&str],
    audit_log_uuid: &str,
    request_id: &str,
) -> ReplaceAdminAiResourceHierarchyCommand {
    let vendor_resource_code = "channel.relay.vendor.1".to_owned();
    let mut nodes = Vec::new();
    let mut vendor_members = Vec::new();
    for (index, model_id) in model_ids.iter().enumerate() {
        let resource_code = format!("channel.relay.model.1.{}", index + 1);
        nodes.push(AdminAiResourceHierarchyNodeCommand {
            resource_uuid: format!("{request_id}-model-{}", index + 1),
            member_uuids: Vec::new(),
            resource_code: resource_code.clone(),
            resource_type: "model".to_owned(),
            display_name: (*model_id).to_owned(),
            vendor_code: Some("openai".to_owned()),
            modality_code: None,
            api_endpoint_code: None,
            catalog_key: Some(format!("openai/{model_id}")),
            model: Some((*model_id).to_owned()),
            provider_native_model: Some((*model_id).to_owned()),
            access_channel_kind: None,
            base_url: None,
            default_vendor_code: None,
            default_model_id: None,
            supported_agent_provider_ids: Vec::new(),
            context_tokens: None,
            max_output_tokens: None,
            tool_call_rounds: None,
            supports_multimodal: None,
            description: None,
            composition_mode: "single".to_owned(),
            status: "active".to_owned(),
            sort_order: None,
            members: Vec::new(),
        });
        vendor_members.push(AdminAiResourceMemberCommand {
            member_resource_code: resource_code,
            member_role: "model".to_owned(),
            required: true,
            sort_order: Some(index as i64),
        });
    }
    nodes.push(AdminAiResourceHierarchyNodeCommand {
        resource_uuid: format!("{request_id}-vendor"),
        member_uuids: vendor_members
            .iter()
            .enumerate()
            .map(|(index, _)| format!("{request_id}-vendor-member-{index}"))
            .collect(),
        resource_code: vendor_resource_code.clone(),
        resource_type: "vendor".to_owned(),
        display_name: "OpenAI".to_owned(),
        vendor_code: Some("openai".to_owned()),
        modality_code: None,
        api_endpoint_code: None,
        catalog_key: None,
        model: None,
        provider_native_model: None,
        access_channel_kind: None,
        base_url: None,
        default_vendor_code: None,
        default_model_id: None,
        supported_agent_provider_ids: Vec::new(),
        context_tokens: None,
        max_output_tokens: None,
        tool_call_rounds: None,
        supports_multimodal: None,
        description: None,
        composition_mode: "all".to_owned(),
        status: "active".to_owned(),
        sort_order: None,
        members: vendor_members,
    });
    nodes.push(AdminAiResourceHierarchyNodeCommand {
        resource_uuid: format!("{request_id}-root"),
        member_uuids: vec![format!("{request_id}-root-member")],
        resource_code: "channel.relay".to_owned(),
        resource_type: "model_access_channel".to_owned(),
        display_name: "Relay".to_owned(),
        vendor_code: None,
        modality_code: None,
        api_endpoint_code: None,
        catalog_key: None,
        model: None,
        provider_native_model: None,
        access_channel_kind: Some("relay".to_owned()),
        base_url: Some("https://relay.example.test/v1".to_owned()),
        default_vendor_code: Some("openai".to_owned()),
        default_model_id: model_ids.first().map(|value| (*value).to_owned()),
        supported_agent_provider_ids: vec!["codex".to_owned(), "gemini".to_owned()],
        context_tokens: None,
        max_output_tokens: None,
        tool_call_rounds: None,
        supports_multimodal: None,
        description: Some("Shared relay".to_owned()),
        composition_mode: "all".to_owned(),
        status: "active".to_owned(),
        sort_order: None,
        members: vec![AdminAiResourceMemberCommand {
            member_resource_code: vendor_resource_code,
            member_role: "vendor".to_owned(),
            required: true,
            sort_order: Some(0),
        }],
    });
    ReplaceAdminAiResourceHierarchyCommand {
        subject: subject(),
        root_resource_code: "channel.relay".to_owned(),
        owned_resource_code_prefixes: vec![
            "channel.relay.vendor.".to_owned(),
            "channel.relay.model.".to_owned(),
        ],
        nodes,
        audit_log_uuid: audit_log_uuid.to_owned(),
        request_id: request_id.to_owned(),
        requested_at: REQUESTED_AT.to_owned(),
    }
}

async fn active_hierarchy_resource_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(1) FROM ai_resource WHERE status = 1 AND deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .expect("count active hierarchy resources")
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

const HIERARCHY_TEST_SCHEMA: &[&str] = &[
    r#"
    CREATE TABLE ai_resource (
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
        metadata TEXT NOT NULL DEFAULT '{}',
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
        sort_order INTEGER,
        UNIQUE (tenant_id, organization_id, resource_code)
    )
    "#,
    r#"
    CREATE TABLE ai_resource_group (
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
        metadata TEXT NOT NULL DEFAULT '{}',
        group_code TEXT NOT NULL,
        group_name TEXT NOT NULL,
        group_type TEXT,
        selection_mode TEXT,
        description TEXT,
        sort_order INTEGER,
        UNIQUE (tenant_id, organization_id, group_code)
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

#[tokio::test]
async fn initialize_schema_creates_resource_tables() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory SQLite");
    let store = SqliteAdminAiResourceStore::new(pool.clone());
    store.initialize_schema().await.expect("initialize schema");
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .expect("list tables");
    for expected in ["ai_resource", "ai_resource_group", "ai_resource_group_item", "ops_audit_log"] {
        assert!(tables.contains(&expected.to_owned()), "missing table {expected}: {tables:?}");
    }
    // Idempotent on repeat startup.
    store.initialize_schema().await.expect("re-initialize schema");
}
