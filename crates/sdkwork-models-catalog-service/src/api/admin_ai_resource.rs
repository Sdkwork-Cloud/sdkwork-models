use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::{get, patch, put};
use axum::Router;
use sdkwork_cloudrouter_http::TrustedRequestSubject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::api::page_info::{offset_page_info, ApiPageInfo};
use crate::api::request_id::{generate_server_request_id, RequestIdError};
use sdkwork_utils_rust::SdkWorkResultCode;

use crate::api::response::{
    finish_no_content, finish_success, finish_success_with_status, problem_for,
};
use crate::application::EntityUuidGenerator;
use crate::domain::DomainError;
use crate::ports::{
    AdminAiResourceGroupItem, AdminAiResourceGroupMemberCommand, AdminAiResourceGroupResourceItem,
    AdminAiResourceItem, AdminAiResourceMemberCommand, AdminAiResourceMemberItem,
    AdminAiResourceStore, AdminAiResourceSubject, CreateAdminAiResourceCommand,
    CreateAdminAiResourceGroupCommand, DeleteAdminAiResourceGroupCommand,
    DeleteAdminAiResourceGroupMemberCommand, ListAdminAiResourceGroupResourcesQuery,
    ListAdminAiResourceGroupsQuery, ListAdminAiResourcesQuery, UpdateAdminAiResourceCommand,
    UpdateAdminAiResourceGroupCommand, UpsertAdminAiResourceGroupMemberCommand,
};
use sdkwork_web_core::WebRequestContext;

const MAX_RESOURCE_CODE_LEN: usize = 192;
const MAX_RESOURCE_TYPE_LEN: usize = 64;
const MAX_DISPLAY_NAME_LEN: usize = 128;
const MAX_VENDOR_CODE_LEN: usize = 64;
const MAX_MODALITY_CODE_LEN: usize = 64;
const MAX_API_ENDPOINT_CODE_LEN: usize = 128;
const MAX_CATALOG_KEY_LEN: usize = 256;
const MAX_MODEL_LEN: usize = 128;
const MAX_PROVIDER_NATIVE_MODEL_LEN: usize = 256;
const MAX_BASE_URL_LEN: usize = 2_048;
const MAX_COMPOSITION_MODE_LEN: usize = 32;
const MAX_GROUP_CODE_LEN: usize = 128;
const MAX_GROUP_NAME_LEN: usize = 128;
const MAX_GROUP_TYPE_LEN: usize = 64;
const MAX_SELECTION_MODE_LEN: usize = 32;
const MAX_DESCRIPTION_LEN: usize = 512;
const MAX_MEMBERS: usize = 512;
const MAX_LIST_SEARCH_LEN: usize = 256;
const DEFAULT_LIST_PAGE_SIZE: i64 = 20;
const MAX_LIST_PAGE_SIZE: i64 = 200;
const CANONICAL_AGENT_PROVIDER_IDS: &[&str] = &[
    "codex",
    "claude-code",
    "gemini",
    "opencode",
    "openclaw",
    "hermes",
];

#[derive(Clone)]
struct AdminAiResourceState {
    store: Arc<dyn AdminAiResourceStore + Send + Sync>,
    entity_uuid_generator: Arc<dyn EntityUuidGenerator + Send + Sync>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourcesResponse {
    items: Vec<AdminAiResourceItemResponse>,
    page_info: ApiPageInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceItemResponse {
    id: String,
    resource_code: String,
    resource_type: String,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    vendor_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modality_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_endpoint_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_native_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_channel_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_vendor_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_model_id: Option<String>,
    supported_agent_provider_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capability: Option<String>,
    capabilities: Vec<String>,
    composition_mode: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<String>,
    members: Vec<AdminAiResourceMemberResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceMemberResponse {
    parent_resource_code: String,
    member_resource_code: String,
    member_role: String,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceItemEnvelope {
    item: AdminAiResourceItemResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupsResponse {
    items: Vec<AdminAiResourceGroupItemResponse>,
    page_info: ApiPageInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupItemEnvelope {
    item: AdminAiResourceGroupItemResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupItemResponse {
    id: String,
    group_code: String,
    group_name: String,
    group_type: String,
    selection_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    vendor_codes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capability: Option<String>,
    capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<String>,
    status: String,
    resource_count: String,
    dynamic: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupResourcesResponse {
    items: Vec<AdminAiResourceGroupResourceItemResponse>,
    page_info: ApiPageInfo,
}

#[derive(Debug, Default, Deserialize)]
struct AiResourceListQuery {
    q: Option<String>,
    resource_type: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
struct AiResourceGroupListQuery {
    q: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupResourceItemResponse {
    id: String,
    resource_code: String,
    resource_type: String,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    vendor_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modality_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_endpoint_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_native_model: Option<String>,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<String>,
    member_role: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupResourceItemEnvelope {
    item: AdminAiResourceGroupResourceItemResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AiResourceMemberRequest {
    member_resource_code: Option<String>,
    member_role: Option<String>,
    required: Option<bool>,
    sort_order: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AiResourceCreateRequest {
    resource_code: Option<String>,
    resource_type: Option<String>,
    display_name: Option<String>,
    vendor_code: Option<String>,
    modality_code: Option<String>,
    api_endpoint_code: Option<String>,
    catalog_key: Option<String>,
    model: Option<String>,
    provider_native_model: Option<String>,
    access_channel_kind: Option<String>,
    base_url: Option<String>,
    default_vendor_code: Option<String>,
    default_model_id: Option<String>,
    supported_agent_provider_ids: Option<Vec<String>>,
    description: Option<String>,
    composition_mode: Option<String>,
    status: Option<String>,
    sort_order: Option<Value>,
    members: Option<Vec<AiResourceMemberRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AiResourceUpdateRequest {
    resource_code: Option<String>,
    resource_type: Option<String>,
    display_name: Option<String>,
    vendor_code: Option<Option<String>>,
    modality_code: Option<Option<String>>,
    api_endpoint_code: Option<Option<String>>,
    catalog_key: Option<Option<String>>,
    model: Option<Option<String>>,
    provider_native_model: Option<Option<String>>,
    access_channel_kind: Option<String>,
    base_url: Option<String>,
    default_vendor_code: Option<String>,
    default_model_id: Option<String>,
    supported_agent_provider_ids: Option<Vec<String>>,
    description: Option<Option<String>>,
    composition_mode: Option<String>,
    status: Option<String>,
    sort_order: Option<Option<Value>>,
    members: Option<Vec<AiResourceMemberRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AiResourceGroupMemberRequest {
    resource_code: Option<String>,
    item_role: Option<String>,
    sort_order: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AiResourceGroupMemberUpdateRequest {
    item_role: Option<String>,
    sort_order: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AiResourceGroupCreateRequest {
    group_code: Option<String>,
    group_name: Option<String>,
    group_type: Option<String>,
    selection_mode: Option<String>,
    description: Option<String>,
    sort_order: Option<Value>,
    status: Option<String>,
    members: Option<Vec<AiResourceGroupMemberRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AiResourceGroupUpdateRequest {
    group_code: Option<String>,
    group_name: Option<String>,
    group_type: Option<String>,
    selection_mode: Option<String>,
    description: Option<Option<String>>,
    sort_order: Option<Option<Value>>,
    status: Option<String>,
    members: Option<Vec<AiResourceGroupMemberRequest>>,
}

#[derive(Debug)]
enum AiResourceCommandBuildError {
    BadRequest(String),
    System(DomainError),
}

pub fn admin_ai_resource_router_with_store(
    store: Arc<dyn AdminAiResourceStore + Send + Sync>,
    entity_uuid_generator: Arc<dyn EntityUuidGenerator + Send + Sync>,
) -> Router {
    Router::new()
        .route(
            "/backend/v3/api/ai/resources",
            get(fetch_ai_resources).post(create_ai_resource),
        )
        .route(
            "/backend/v3/api/ai/resources/{resource_id}",
            put(update_ai_resource),
        )
        .route(
            "/backend/v3/api/ai/resource_groups",
            get(fetch_ai_resource_groups).post(create_ai_resource_group),
        )
        .route(
            "/backend/v3/api/ai/resource_groups/{group_id_or_code}/resources",
            get(fetch_ai_resource_group_resources),
        )
        .route(
            "/backend/v3/api/ai/resource_groups/{group_id}/resources/{resource_code}",
            put(upsert_ai_resource_group_member).delete(delete_ai_resource_group_member),
        )
        .route(
            "/backend/v3/api/ai/resource_groups/{group_id}",
            patch(update_ai_resource_group).delete(delete_ai_resource_group),
        )
        .with_state(AdminAiResourceState {
            store,
            entity_uuid_generator,
        })
}

async fn fetch_ai_resources(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Query(query): Query<AiResourceListQuery>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
) -> Response {
    let subject = map_subject(trusted);
    let q = match validate_list_search_query(query.q) {
        Ok(value) => value,
        Err(message) => return bad_request(&ctx, message),
    };
    let resource_type = match query.resource_type.map(normalize_resource_type).transpose() {
        Ok(value) => value,
        Err(error) => return command_build_error_response(&ctx, error),
    };
    let (page_no, page_size, offset) = match validate_page_query(query.page, query.page_size) {
        Ok(value) => value,
        Err(message) => return bad_request(&ctx, message),
    };

    match state
        .store
        .list_ai_resources(ListAdminAiResourcesQuery {
            subject,
            q,
            resource_type,
            status: None,
            access_channel_kind: None,
            vendor_code: None,
            agent_provider_id: None,
            require_valid_access_channel_metadata: false,
            limit: Some(page_size),
            offset: Some(offset),
        })
        .await
    {
        Ok(items) => finish_success(
            &ctx,
            AdminAiResourcesResponse {
                items: items.items.into_iter().map(to_item_response).collect(),
                page_info: offset_page_info(page_no, page_size, items.total_count),
            },
        ),
        Err(error) => {
            ai_resource_system_response(&ctx, "AI resource read model is unavailable", error)
        }
    }
}

async fn fetch_ai_resource_groups(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Query(query): Query<AiResourceGroupListQuery>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
) -> Response {
    let subject = map_subject(trusted);
    let q = match validate_list_search_query(query.q) {
        Ok(value) => value,
        Err(message) => return bad_request(&ctx, message),
    };
    let (page_no, page_size, offset) = match validate_page_query(query.page, query.page_size) {
        Ok(value) => value,
        Err(message) => return bad_request(&ctx, message),
    };

    match state
        .store
        .list_ai_resource_groups(ListAdminAiResourceGroupsQuery {
            subject,
            q,
            limit: Some(page_size),
            offset: Some(offset),
        })
        .await
    {
        Ok(items) => finish_success(
            &ctx,
            AdminAiResourceGroupsResponse {
                items: items.items.into_iter().map(to_group_response).collect(),
                page_info: offset_page_info(page_no, page_size, items.total_count),
            },
        ),
        Err(error) => {
            ai_resource_system_response(&ctx, "AI resource group read model is unavailable", error)
        }
    }
}

async fn fetch_ai_resource_group_resources(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Path(group_id_or_code): Path<String>,
    Query(query): Query<AiResourceGroupListQuery>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
) -> Response {
    let subject = map_subject(trusted);
    let group_id_or_code = group_id_or_code.trim().to_owned();
    if group_id_or_code.is_empty() {
        return bad_request(&ctx, "AI resource group id or code is required".to_owned());
    }
    let q = match validate_list_search_query(query.q) {
        Ok(value) => value,
        Err(message) => return bad_request(&ctx, message),
    };
    let (page_no, page_size, offset) = match validate_page_query(query.page, query.page_size) {
        Ok(value) => value,
        Err(message) => return bad_request(&ctx, message),
    };

    match state
        .store
        .list_ai_resource_group_resources(ListAdminAiResourceGroupResourcesQuery {
            subject,
            group_id_or_code,
            q,
            limit: Some(page_size),
            offset: Some(offset),
        })
        .await
    {
        Ok(items) => finish_success(
            &ctx,
            AdminAiResourceGroupResourcesResponse {
                page_info: offset_page_info(page_no, page_size, items.total_count),
                items: items
                    .items
                    .into_iter()
                    .map(to_group_resource_response)
                    .collect(),
            },
        ),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) => ai_resource_system_response(
            &ctx,
            "AI resource group resource read model is unavailable",
            error,
        ),
    }
}

async fn upsert_ai_resource_group_member(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Path((group_id, resource_code)): Path<(String, String)>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    let subject = map_subject(trusted);
    let group_id = match parse_positive_id(&group_id, "AI resource group id") {
        Ok(group_id) => group_id,
        Err(message) => return bad_request(&ctx, message),
    };
    let resource_code = match required_resource_code(
        Some(resource_code),
        "resourceCode",
        "AI resource code",
        MAX_RESOURCE_CODE_LEN,
    ) {
        Ok(resource_code) => resource_code,
        Err(error) => return command_build_error_response(&ctx, error),
    };
    let request = match parse_json_body::<AiResourceGroupMemberUpdateRequest>(
        &body,
        "AI resource group member update",
    ) {
        Ok(request) => request,
        Err(message) => return bad_request(&ctx, message),
    };
    let command = match build_group_member_upsert_command(
        state.clone(),
        subject,
        group_id,
        resource_code,
        request,
    ) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(&ctx, error),
    };

    match state.store.upsert_ai_resource_group_member(command).await {
        Ok(Some(item)) => finish_success(
            &ctx,
            AdminAiResourceGroupResourceItemEnvelope {
                item: to_group_resource_response(item),
            },
        ),
        Ok(None) => not_found_response(&ctx, "AI resource group was not found"),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(&ctx, error),
        Err(error) => ai_resource_system_response(
            &ctx,
            "AI resource group member command store is unavailable",
            error,
        ),
    }
}

async fn delete_ai_resource_group_member(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Path((group_id, resource_code)): Path<(String, String)>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
) -> Response {
    let subject = map_subject(trusted);
    let group_id = match parse_positive_id(&group_id, "AI resource group id") {
        Ok(group_id) => group_id,
        Err(message) => return bad_request(&ctx, message),
    };
    let resource_code = match required_resource_code(
        Some(resource_code),
        "resourceCode",
        "AI resource code",
        MAX_RESOURCE_CODE_LEN,
    ) {
        Ok(resource_code) => resource_code,
        Err(error) => return command_build_error_response(&ctx, error),
    };
    let command =
        match build_group_member_delete_command(state.clone(), subject, group_id, resource_code) {
            Ok(command) => command,
            Err(error) => return command_build_error_response(&ctx, error),
        };

    match state.store.delete_ai_resource_group_member(command).await {
        Ok(true) => finish_no_content(&ctx),
        Ok(false) => not_found_response(&ctx, "AI resource group was not found"),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(&ctx, error),
        Err(error) => ai_resource_system_response(
            &ctx,
            "AI resource group member command store is unavailable",
            error,
        ),
    }
}

async fn create_ai_resource(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    let subject = map_subject(trusted);
    let request = match parse_json_body::<AiResourceCreateRequest>(&body, "AI resource") {
        Ok(request) => request,
        Err(message) => return bad_request(&ctx, message),
    };
    let command = match build_create_command(state.clone(), subject, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(&ctx, error),
    };

    match state.store.create_ai_resource(command).await {
        Ok(item) => finish_success_with_status(
            &ctx,
            StatusCode::CREATED,
            AdminAiResourceItemEnvelope {
                item: to_item_response(item),
            },
        ),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(&ctx, error),
        Err(error) => {
            ai_resource_system_response(&ctx, "AI resource command store is unavailable", error)
        }
    }
}

async fn create_ai_resource_group(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    let subject = map_subject(trusted);
    let request = match parse_json_body::<AiResourceGroupCreateRequest>(&body, "AI resource group")
    {
        Ok(request) => request,
        Err(message) => return bad_request(&ctx, message),
    };
    let command = match build_group_create_command(state.clone(), subject, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(&ctx, error),
    };

    match state.store.create_ai_resource_group(command).await {
        Ok(item) => finish_success_with_status(
            &ctx,
            StatusCode::CREATED,
            AdminAiResourceGroupItemEnvelope {
                item: to_group_response(item),
            },
        ),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(&ctx, error),
        Err(error) => ai_resource_system_response(
            &ctx,
            "AI resource group command store is unavailable",
            error,
        ),
    }
}

async fn update_ai_resource_group(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Path(group_id): Path<String>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    let subject = map_subject(trusted);
    let group_id = match parse_positive_id(&group_id, "AI resource group id") {
        Ok(group_id) => group_id,
        Err(message) => return bad_request(&ctx, message),
    };
    let request =
        match parse_json_body::<AiResourceGroupUpdateRequest>(&body, "AI resource group update") {
            Ok(request) => request,
            Err(message) => return bad_request(&ctx, message),
        };
    let command = match build_group_update_command(state.clone(), subject, group_id, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(&ctx, error),
    };

    match state.store.update_ai_resource_group(command).await {
        Ok(Some(item)) => finish_success(
            &ctx,
            AdminAiResourceGroupItemEnvelope {
                item: to_group_response(item),
            },
        ),
        Ok(None) => not_found_response(&ctx, "AI resource group was not found"),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(&ctx, error),
        Err(error) => ai_resource_system_response(
            &ctx,
            "AI resource group command store is unavailable",
            error,
        ),
    }
}

async fn delete_ai_resource_group(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Path(group_id): Path<String>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
) -> Response {
    let subject = map_subject(trusted);
    let group_id = match parse_positive_id(&group_id, "AI resource group id") {
        Ok(group_id) => group_id,
        Err(message) => return bad_request(&ctx, message),
    };
    let command = match build_group_delete_command(state.clone(), subject, group_id) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(&ctx, error),
    };

    match state.store.delete_ai_resource_group(command).await {
        Ok(true) => finish_no_content(&ctx),
        Ok(false) => not_found_response(&ctx, "AI resource group was not found"),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(&ctx, error),
        Err(error) => ai_resource_system_response(
            &ctx,
            "AI resource group command store is unavailable",
            error,
        ),
    }
}

async fn update_ai_resource(
    ctx: WebRequestContext,
    State(state): State<AdminAiResourceState>,
    Path(resource_id): Path<String>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    let subject = map_subject(trusted);
    let resource_id = match parse_positive_id(&resource_id, "AI resource id") {
        Ok(resource_id) => resource_id,
        Err(message) => return bad_request(&ctx, message),
    };
    let request = match parse_json_body::<AiResourceUpdateRequest>(&body, "AI resource update") {
        Ok(request) => request,
        Err(message) => return bad_request(&ctx, message),
    };
    let command = match build_update_command(state.clone(), subject, resource_id, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(&ctx, error),
    };

    match state.store.update_ai_resource(command).await {
        Ok(Some(item)) => finish_success(
            &ctx,
            AdminAiResourceItemEnvelope {
                item: to_item_response(item),
            },
        ),
        Ok(None) => not_found_response(&ctx, "AI resource was not found"),
        Err(error) if error.is_not_found() => not_found_response(&ctx, error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(&ctx, error),
        Err(error) => {
            ai_resource_system_response(&ctx, "AI resource command store is unavailable", error)
        }
    }
}

fn map_subject(trusted: TrustedRequestSubject) -> AdminAiResourceSubject {
    AdminAiResourceSubject {
        tenant_id: trusted.tenant_id,
        organization_id: trusted.organization_id,
        operator_id: trusted.operator_id,
        operator_type: trusted.operator_type,
    }
}

fn to_item_response(item: AdminAiResourceItem) -> AdminAiResourceItemResponse {
    AdminAiResourceItemResponse {
        id: item.id.to_string(),
        resource_code: item.resource_code,
        resource_type: item.resource_type,
        display_name: item.display_name,
        vendor_code: item.vendor_code,
        modality_code: item.modality_code,
        api_endpoint_code: item.api_endpoint_code,
        catalog_key: item.catalog_key,
        model: item.model,
        provider_native_model: item.provider_native_model,
        access_channel_kind: item.access_channel_kind,
        base_url: item.base_url,
        default_vendor_code: item.default_vendor_code,
        default_model_id: item.default_model_id,
        supported_agent_provider_ids: item.supported_agent_provider_ids,
        description: item.description,
        capability: item.capability,
        capabilities: item.capabilities,
        composition_mode: item.composition_mode,
        status: item.status,
        sort_order: item.sort_order.map(|value| value.to_string()),
        members: item.members.into_iter().map(to_member_response).collect(),
    }
}

fn to_member_response(member: AdminAiResourceMemberItem) -> AdminAiResourceMemberResponse {
    AdminAiResourceMemberResponse {
        parent_resource_code: member.parent_resource_code,
        member_resource_code: member.member_resource_code,
        member_role: member.member_role,
        required: member.required,
        sort_order: member.sort_order.map(|value| value.to_string()),
    }
}

fn to_group_response(item: AdminAiResourceGroupItem) -> AdminAiResourceGroupItemResponse {
    AdminAiResourceGroupItemResponse {
        id: item.id.to_string(),
        group_code: item.group_code,
        group_name: item.group_name,
        group_type: item.group_type,
        selection_mode: item.selection_mode,
        description: item.description,
        vendor_codes: item.vendor_codes,
        capability: item.capability,
        capabilities: item.capabilities,
        sort_order: item.sort_order.map(|value| value.to_string()),
        status: item.status,
        resource_count: item.resource_count.to_string(),
        dynamic: item.dynamic,
    }
}

fn to_group_resource_response(
    item: AdminAiResourceGroupResourceItem,
) -> AdminAiResourceGroupResourceItemResponse {
    AdminAiResourceGroupResourceItemResponse {
        id: item.id.to_string(),
        resource_code: item.resource_code,
        resource_type: item.resource_type,
        display_name: item.display_name,
        vendor_code: item.vendor_code,
        modality_code: item.modality_code,
        api_endpoint_code: item.api_endpoint_code,
        catalog_key: item.catalog_key,
        model: item.model,
        provider_native_model: item.provider_native_model,
        status: item.status,
        sort_order: item.sort_order.map(|value| value.to_string()),
        member_role: item.member_role,
    }
}

fn parse_json_body<T: for<'de> Deserialize<'de>>(body: &[u8], label: &str) -> Result<T, String> {
    if body.iter().all(u8::is_ascii_whitespace) {
        return Err(format!("{label} request body is required"));
    }
    serde_json::from_slice(body).map_err(|error| format!("invalid {label} request body: {error}"))
}

fn build_group_create_command(
    state: AdminAiResourceState,
    subject: AdminAiResourceSubject,
    request: AiResourceGroupCreateRequest,
) -> Result<CreateAdminAiResourceGroupCommand, AiResourceCommandBuildError> {
    let selection_mode = normalize_selection_mode(
        optional_text(
            request.selection_mode,
            "selectionMode",
            MAX_SELECTION_MODE_LEN,
        )?
        .unwrap_or_else(|| "manual".to_owned()),
    )?;
    let group_code = required_group_code(request.group_code)?;
    let members = normalize_group_members(request.members.unwrap_or_default())?;
    if is_dynamic_all_group(&group_code, &selection_mode) && !members.is_empty() {
        return Err(AiResourceCommandBuildError::BadRequest(
            "dynamic API groups cannot maintain resource relationships".to_owned(),
        ));
    }
    Ok(CreateAdminAiResourceGroupCommand {
        subject,
        group_uuid: generate_entity_uuid(&state)?,
        member_uuids: generate_entity_uuids(&state, members.len())?,
        audit_log_uuid: generate_entity_uuid(&state)?,
        group_code,
        group_name: required_text(
            request.group_name,
            "groupName",
            "AI resource group name",
            MAX_GROUP_NAME_LEN,
        )?,
        group_type: normalize_group_type(
            optional_text(request.group_type, "groupType", MAX_GROUP_TYPE_LEN)?
                .unwrap_or_else(|| "api_group".to_owned()),
        )?,
        selection_mode,
        description: optional_text(request.description, "description", MAX_DESCRIPTION_LEN)?,
        sort_order: optional_non_negative(request.sort_order.as_ref(), "sortOrder")?,
        status: normalize_status(
            optional_text(request.status, "status", 32)?.unwrap_or_else(|| "active".to_owned()),
        )?,
        members,
        request_id: generate_server_request_id().map_err(request_id_error)?,
        requested_at: current_timestamp_string(),
    })
}

fn build_group_update_command(
    state: AdminAiResourceState,
    subject: AdminAiResourceSubject,
    group_id: i64,
    request: AiResourceGroupUpdateRequest,
) -> Result<UpdateAdminAiResourceGroupCommand, AiResourceCommandBuildError> {
    let members = request.members.map(normalize_group_members).transpose()?;
    let member_count = members.as_ref().map(Vec::len).unwrap_or(0);
    Ok(UpdateAdminAiResourceGroupCommand {
        subject,
        group_id,
        member_uuids: generate_entity_uuids(&state, member_count)?,
        audit_log_uuid: generate_entity_uuid(&state)?,
        group_code: request.group_code.map(group_code_value).transpose()?,
        group_name: request
            .group_name
            .map(|value| {
                required_text(
                    Some(value),
                    "groupName",
                    "AI resource group name",
                    MAX_GROUP_NAME_LEN,
                )
            })
            .transpose()?,
        group_type: request.group_type.map(normalize_group_type).transpose()?,
        selection_mode: request
            .selection_mode
            .map(normalize_selection_mode)
            .transpose()?,
        description: request
            .description
            .map(|value| optional_text(value, "description", MAX_DESCRIPTION_LEN))
            .transpose()?,
        sort_order: request
            .sort_order
            .map(|value| optional_non_negative(value.as_ref(), "sortOrder"))
            .transpose()?,
        status: request.status.map(normalize_status).transpose()?,
        members,
        request_id: generate_server_request_id().map_err(request_id_error)?,
        requested_at: current_timestamp_string(),
    })
}

fn build_group_member_upsert_command(
    state: AdminAiResourceState,
    subject: AdminAiResourceSubject,
    group_id: i64,
    resource_code: String,
    request: AiResourceGroupMemberUpdateRequest,
) -> Result<UpsertAdminAiResourceGroupMemberCommand, AiResourceCommandBuildError> {
    Ok(UpsertAdminAiResourceGroupMemberCommand {
        subject,
        group_id,
        member_uuid: generate_entity_uuid(&state)?,
        audit_log_uuid: generate_entity_uuid(&state)?,
        member: AdminAiResourceGroupMemberCommand {
            resource_code,
            item_role: normalize_group_item_role(
                optional_text(request.item_role, "itemRole", 64)?
                    .unwrap_or_else(|| "included".to_owned()),
            )?,
            sort_order: optional_non_negative(request.sort_order.as_ref(), "sortOrder")?,
        },
        request_id: generate_server_request_id().map_err(request_id_error)?,
        requested_at: current_timestamp_string(),
    })
}

fn build_group_member_delete_command(
    state: AdminAiResourceState,
    subject: AdminAiResourceSubject,
    group_id: i64,
    resource_code: String,
) -> Result<DeleteAdminAiResourceGroupMemberCommand, AiResourceCommandBuildError> {
    Ok(DeleteAdminAiResourceGroupMemberCommand {
        subject,
        group_id,
        resource_code,
        audit_log_uuid: generate_entity_uuid(&state)?,
        request_id: generate_server_request_id().map_err(request_id_error)?,
        requested_at: current_timestamp_string(),
    })
}

fn build_group_delete_command(
    state: AdminAiResourceState,
    subject: AdminAiResourceSubject,
    group_id: i64,
) -> Result<DeleteAdminAiResourceGroupCommand, AiResourceCommandBuildError> {
    Ok(DeleteAdminAiResourceGroupCommand {
        subject,
        group_id,
        audit_log_uuid: generate_entity_uuid(&state)?,
        request_id: generate_server_request_id().map_err(request_id_error)?,
        requested_at: current_timestamp_string(),
    })
}

fn build_create_command(
    state: AdminAiResourceState,
    subject: AdminAiResourceSubject,
    request: AiResourceCreateRequest,
) -> Result<CreateAdminAiResourceCommand, AiResourceCommandBuildError> {
    let members = normalize_members(request.members.unwrap_or_default())?;
    let resource_type = normalize_resource_type(required_text(
        request.resource_type,
        "resourceType",
        "AI resource type",
        MAX_RESOURCE_TYPE_LEN,
    )?)?;
    let is_access_channel = resource_type == "model_access_channel";
    if !is_access_channel
        && (request.default_vendor_code.is_some() || request.default_model_id.is_some())
    {
        return Err(AiResourceCommandBuildError::BadRequest(
            "default model fields require resourceType model_access_channel".to_owned(),
        ));
    }
    let access_channel_kind = normalize_access_channel_kind(
        request.access_channel_kind,
        is_access_channel,
        is_access_channel,
    )?;
    let base_url = normalize_base_url(request.base_url, is_access_channel, is_access_channel)?;
    let supported_agent_provider_ids = normalize_supported_agent_provider_ids(
        request.supported_agent_provider_ids,
        is_access_channel,
        is_access_channel,
    )?;
    Ok(CreateAdminAiResourceCommand {
        subject,
        resource_uuid: generate_entity_uuid(&state)?,
        member_uuids: generate_entity_uuids(&state, members.len())?,
        audit_log_uuid: generate_entity_uuid(&state)?,
        resource_code: required_resource_code(
            request.resource_code,
            "resourceCode",
            "AI resource code",
            MAX_RESOURCE_CODE_LEN,
        )?,
        resource_type,
        display_name: required_text(
            request.display_name,
            "displayName",
            "AI resource display name",
            MAX_DISPLAY_NAME_LEN,
        )?,
        vendor_code: optional_code(request.vendor_code, "vendorCode", MAX_VENDOR_CODE_LEN)?,
        modality_code: optional_code(request.modality_code, "modalityCode", MAX_MODALITY_CODE_LEN)?,
        api_endpoint_code: optional_code(
            request.api_endpoint_code,
            "apiEndpointCode",
            MAX_API_ENDPOINT_CODE_LEN,
        )?,
        catalog_key: optional_catalog_key(request.catalog_key)?,
        model: optional_visible_text(request.model, "model", MAX_MODEL_LEN)?,
        provider_native_model: optional_visible_text(
            request.provider_native_model,
            "providerNativeModel",
            MAX_PROVIDER_NATIVE_MODEL_LEN,
        )?,
        access_channel_kind,
        base_url,
        default_vendor_code: optional_code(
            request.default_vendor_code,
            "defaultVendorCode",
            MAX_VENDOR_CODE_LEN,
        )?,
        default_model_id: optional_visible_text(
            request.default_model_id,
            "defaultModelId",
            MAX_MODEL_LEN,
        )?,
        supported_agent_provider_ids,
        description: optional_text(request.description, "description", MAX_DESCRIPTION_LEN)?,
        composition_mode: normalize_composition_mode(
            optional_text(
                request.composition_mode,
                "compositionMode",
                MAX_COMPOSITION_MODE_LEN,
            )?
            .unwrap_or_else(|| "single".to_owned()),
        )?,
        status: normalize_status(
            optional_text(request.status, "status", 32)?.unwrap_or_else(|| "active".to_owned()),
        )?,
        sort_order: optional_non_negative(request.sort_order.as_ref(), "sortOrder")?,
        members,
        request_id: generate_server_request_id().map_err(request_id_error)?,
        requested_at: current_timestamp_string(),
    })
}

fn normalize_group_members(
    members: Vec<AiResourceGroupMemberRequest>,
) -> Result<Vec<AdminAiResourceGroupMemberCommand>, AiResourceCommandBuildError> {
    if members.len() > MAX_MEMBERS {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "members must contain at most {MAX_MEMBERS} items"
        )));
    }
    let mut normalized = Vec::with_capacity(members.len());
    for member in members {
        let resource_code = required_resource_code(
            member.resource_code,
            "resourceCode",
            "AI resource code",
            MAX_RESOURCE_CODE_LEN,
        )?;
        if normalized
            .iter()
            .any(|existing: &AdminAiResourceGroupMemberCommand| {
                existing.resource_code == resource_code
            })
        {
            continue;
        }
        normalized.push(AdminAiResourceGroupMemberCommand {
            resource_code,
            item_role: normalize_group_item_role(
                optional_text(member.item_role, "itemRole", 64)?
                    .unwrap_or_else(|| "included".to_owned()),
            )?,
            sort_order: optional_non_negative(member.sort_order.as_ref(), "member.sortOrder")?,
        });
    }
    Ok(normalized)
}

fn build_update_command(
    state: AdminAiResourceState,
    subject: AdminAiResourceSubject,
    resource_id: i64,
    request: AiResourceUpdateRequest,
) -> Result<UpdateAdminAiResourceCommand, AiResourceCommandBuildError> {
    let members = request.members.map(normalize_members).transpose()?;
    let member_count = members.as_ref().map(Vec::len).unwrap_or(0);
    let resource_type = request
        .resource_type
        .map(|value| {
            normalize_resource_type(required_text(
                Some(value),
                "resourceType",
                "AI resource type",
                MAX_RESOURCE_TYPE_LEN,
            )?)
        })
        .transpose()?;
    let explicitly_access_channel = resource_type.as_deref() == Some("model_access_channel");
    let explicitly_other_resource = resource_type
        .as_deref()
        .is_some_and(|value| value != "model_access_channel");
    if explicitly_other_resource
        && (request.access_channel_kind.is_some()
            || request.base_url.is_some()
            || request.default_vendor_code.is_some()
            || request.default_model_id.is_some()
            || request.supported_agent_provider_ids.is_some())
    {
        return Err(AiResourceCommandBuildError::BadRequest(
            "access channel fields require resourceType model_access_channel".to_owned(),
        ));
    }
    let access_channel_kind = normalize_access_channel_kind(
        request.access_channel_kind,
        !explicitly_other_resource,
        explicitly_access_channel,
    )?;
    let base_url = normalize_base_url(
        request.base_url,
        !explicitly_other_resource,
        explicitly_access_channel,
    )?;
    let supported_agent_provider_ids = match request.supported_agent_provider_ids {
        Some(values) => Some(normalize_agent_provider_ids(values, false)?),
        None if explicitly_access_channel => Some(canonical_agent_provider_ids()),
        None => None,
    };
    Ok(UpdateAdminAiResourceCommand {
        subject,
        resource_id,
        member_uuids: generate_entity_uuids(&state, member_count)?,
        audit_log_uuid: generate_entity_uuid(&state)?,
        resource_code: request
            .resource_code
            .map(|value| resource_code_value(value, "resourceCode", MAX_RESOURCE_CODE_LEN))
            .transpose()?,
        resource_type,
        display_name: request
            .display_name
            .map(|value| {
                required_text(
                    Some(value),
                    "displayName",
                    "AI resource display name",
                    MAX_DISPLAY_NAME_LEN,
                )
            })
            .transpose()?,
        vendor_code: request
            .vendor_code
            .map(|value| optional_code(value, "vendorCode", MAX_VENDOR_CODE_LEN))
            .transpose()?,
        modality_code: request
            .modality_code
            .map(|value| optional_code(value, "modalityCode", MAX_MODALITY_CODE_LEN))
            .transpose()?,
        api_endpoint_code: request
            .api_endpoint_code
            .map(|value| optional_code(value, "apiEndpointCode", MAX_API_ENDPOINT_CODE_LEN))
            .transpose()?,
        catalog_key: request.catalog_key.map(optional_catalog_key).transpose()?,
        model: request
            .model
            .map(|value| optional_visible_text(value, "model", MAX_MODEL_LEN))
            .transpose()?,
        provider_native_model: request
            .provider_native_model
            .map(|value| {
                optional_visible_text(value, "providerNativeModel", MAX_PROVIDER_NATIVE_MODEL_LEN)
            })
            .transpose()?,
        access_channel_kind,
        base_url,
        default_vendor_code: request
            .default_vendor_code
            .map(|value| resource_code_value(value, "defaultVendorCode", MAX_VENDOR_CODE_LEN))
            .transpose()?,
        default_model_id: request
            .default_model_id
            .map(|value| {
                required_text(
                    Some(value),
                    "defaultModelId",
                    "default model id",
                    MAX_MODEL_LEN,
                )
            })
            .transpose()?,
        supported_agent_provider_ids,
        description: request
            .description
            .map(|value| optional_text(value, "description", MAX_DESCRIPTION_LEN))
            .transpose()?,
        composition_mode: request
            .composition_mode
            .map(normalize_composition_mode)
            .transpose()?,
        status: request.status.map(normalize_status).transpose()?,
        sort_order: request
            .sort_order
            .map(|value| optional_non_negative(value.as_ref(), "sortOrder"))
            .transpose()?,
        members,
        request_id: generate_server_request_id().map_err(request_id_error)?,
        requested_at: current_timestamp_string(),
    })
}

fn normalize_members(
    members: Vec<AiResourceMemberRequest>,
) -> Result<Vec<AdminAiResourceMemberCommand>, AiResourceCommandBuildError> {
    if members.len() > MAX_MEMBERS {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "members must contain at most {MAX_MEMBERS} items"
        )));
    }
    let mut normalized = Vec::with_capacity(members.len());
    for member in members {
        let member_resource_code = required_resource_code(
            member.member_resource_code,
            "memberResourceCode",
            "member resource code",
            MAX_RESOURCE_CODE_LEN,
        )?;
        if normalized
            .iter()
            .any(|existing: &AdminAiResourceMemberCommand| {
                existing.member_resource_code == member_resource_code
            })
        {
            continue;
        }
        normalized.push(AdminAiResourceMemberCommand {
            member_resource_code,
            member_role: normalize_member_role(
                optional_text(member.member_role, "memberRole", 64)?
                    .unwrap_or_else(|| "included".to_owned()),
            )?,
            required: member.required.unwrap_or(true),
            sort_order: optional_non_negative(member.sort_order.as_ref(), "member.sortOrder")?,
        });
    }
    Ok(normalized)
}

fn required_resource_code(
    value: Option<String>,
    field_name: &str,
    label: &str,
    max_len: usize,
) -> Result<String, AiResourceCommandBuildError> {
    resource_code_value(
        required_text(value, field_name, label, max_len)?,
        field_name,
        max_len,
    )
}

fn required_group_code(value: Option<String>) -> Result<String, AiResourceCommandBuildError> {
    group_code_value(required_text(
        value,
        "groupCode",
        "AI resource group code",
        MAX_GROUP_CODE_LEN,
    )?)
}

fn group_code_value(value: String) -> Result<String, AiResourceCommandBuildError> {
    resource_code_value(value, "groupCode", MAX_GROUP_CODE_LEN)
}

fn resource_code_value(
    value: String,
    field_name: &str,
    max_len: usize,
) -> Result<String, AiResourceCommandBuildError> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() || value.len() > max_len {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "{field_name} is invalid"
        )));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "{field_name} may only contain letters, numbers, ., -, and _"
        )));
    }
    Ok(value)
}

fn optional_code(
    value: Option<String>,
    field_name: &str,
    max_len: usize,
) -> Result<Option<String>, AiResourceCommandBuildError> {
    value
        .map(|value| resource_code_value(value, field_name, max_len))
        .transpose()
}

fn optional_catalog_key(
    value: Option<String>,
) -> Result<Option<String>, AiResourceCommandBuildError> {
    optional_visible_text(value, "catalogKey", MAX_CATALOG_KEY_LEN)
}

fn required_text(
    value: Option<String>,
    field_name: &str,
    label: &str,
    max_len: usize,
) -> Result<String, AiResourceCommandBuildError> {
    let value = optional_text(value, field_name, max_len)?;
    value.ok_or_else(|| AiResourceCommandBuildError::BadRequest(format!("{label} is required")))
}

fn optional_text(
    value: Option<String>,
    field_name: &str,
    max_len: usize,
) -> Result<Option<String>, AiResourceCommandBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max_len {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "{field_name} must be at most {max_len} characters"
        )));
    }
    Ok(Some(value.to_owned()))
}

fn optional_visible_text(
    value: Option<String>,
    field_name: &str,
    max_len: usize,
) -> Result<Option<String>, AiResourceCommandBuildError> {
    let value = optional_text(value, field_name, max_len)?;
    if let Some(value) = value.as_deref() {
        if !value.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(AiResourceCommandBuildError::BadRequest(format!(
                "{field_name} must contain only visible ASCII characters"
            )));
        }
    }
    Ok(value)
}

fn validate_page_query(
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<(i64, i64, i64), String> {
    let page = page.unwrap_or(1);
    if page < 1 {
        return Err("page must be greater than or equal to 1".to_owned());
    }
    let page_size = page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE);
    if !(1..=MAX_LIST_PAGE_SIZE).contains(&page_size) {
        return Err(format!(
            "page_size must be between 1 and {MAX_LIST_PAGE_SIZE}"
        ));
    }
    let offset = page
        .checked_sub(1)
        .and_then(|value| value.checked_mul(page_size))
        .ok_or_else(|| "page is too large".to_owned())?;
    Ok((page, page_size, offset))
}

fn validate_list_search_query(q: Option<String>) -> Result<Option<String>, String> {
    let Some(q) = q else {
        return Ok(None);
    };
    let q = q.trim();
    if q.is_empty() {
        return Ok(None);
    }
    if q.chars().count() > MAX_LIST_SEARCH_LEN {
        return Err(format!(
            "q must not exceed {MAX_LIST_SEARCH_LEN} characters"
        ));
    }
    Ok(Some(q.to_owned()))
}

fn normalize_resource_type(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "vendor"
        | "modality"
        | "api_endpoint"
        | "model"
        | "model_api"
        | "model_access_channel"
        | "bundle" => {
            Ok(value.trim().to_ascii_lowercase())
        }
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "resourceType must be one of vendor, modality, api_endpoint, model, model_api, model_access_channel, bundle".to_owned(),
        )),
    }
}

fn normalize_access_channel_kind(
    value: Option<String>,
    access_channel_fields_allowed: bool,
    required: bool,
) -> Result<Option<String>, AiResourceCommandBuildError> {
    if !access_channel_fields_allowed && value.is_some() {
        return Err(AiResourceCommandBuildError::BadRequest(
            "accessChannelKind requires resourceType model_access_channel".to_owned(),
        ));
    }
    let normalized =
        optional_text(value, "accessChannelKind", 16)?.map(|value| value.to_ascii_lowercase());
    if required && normalized.is_none() {
        return Err(AiResourceCommandBuildError::BadRequest(
            "accessChannelKind is required for model_access_channel".to_owned(),
        ));
    }
    if normalized
        .as_deref()
        .is_some_and(|value| !matches!(value, "official" | "relay"))
    {
        return Err(AiResourceCommandBuildError::BadRequest(
            "accessChannelKind must be one of official, relay".to_owned(),
        ));
    }
    Ok(normalized)
}

fn normalize_base_url(
    value: Option<String>,
    access_channel_fields_allowed: bool,
    required: bool,
) -> Result<Option<String>, AiResourceCommandBuildError> {
    if !access_channel_fields_allowed && value.is_some() {
        return Err(AiResourceCommandBuildError::BadRequest(
            "baseUrl requires resourceType model_access_channel".to_owned(),
        ));
    }
    let Some(value) = optional_text(value, "baseUrl", MAX_BASE_URL_LEN)? else {
        if required {
            return Err(AiResourceCommandBuildError::BadRequest(
                "baseUrl is required for model_access_channel".to_owned(),
            ));
        }
        return Ok(None);
    };
    let parsed = Url::parse(&value).map_err(|_| {
        AiResourceCommandBuildError::BadRequest(
            "baseUrl must be an absolute HTTP(S) URL".to_owned(),
        )
    })?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err(AiResourceCommandBuildError::BadRequest(
            "baseUrl must be an absolute HTTP(S) URL".to_owned(),
        ));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(AiResourceCommandBuildError::BadRequest(
            "baseUrl must not contain credentials".to_owned(),
        ));
    }
    Ok(Some(parsed.to_string()))
}

fn normalize_supported_agent_provider_ids(
    value: Option<Vec<String>>,
    access_channel_fields_allowed: bool,
    default_all: bool,
) -> Result<Vec<String>, AiResourceCommandBuildError> {
    if !access_channel_fields_allowed && value.is_some() {
        return Err(AiResourceCommandBuildError::BadRequest(
            "supportedAgentProviderIds requires resourceType model_access_channel".to_owned(),
        ));
    }
    match value {
        Some(values) => normalize_agent_provider_ids(values, false),
        None if default_all => Ok(canonical_agent_provider_ids()),
        None => Ok(Vec::new()),
    }
}

fn normalize_agent_provider_ids(
    values: Vec<String>,
    allow_empty: bool,
) -> Result<Vec<String>, AiResourceCommandBuildError> {
    let mut normalized = values
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() && !allow_empty {
        return Err(AiResourceCommandBuildError::BadRequest(
            "supportedAgentProviderIds must contain at least one provider".to_owned(),
        ));
    }
    if let Some(unknown) = normalized
        .iter()
        .find(|value| !CANONICAL_AGENT_PROVIDER_IDS.contains(&value.as_str()))
    {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "supportedAgentProviderIds contains an unsupported provider: {unknown}"
        )));
    }
    Ok(normalized)
}

fn canonical_agent_provider_ids() -> Vec<String> {
    CANONICAL_AGENT_PROVIDER_IDS
        .iter()
        .map(|value| (*value).to_owned())
        .collect()
}

fn normalize_composition_mode(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "single" | "any" | "all" => Ok(value.trim().to_ascii_lowercase()),
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "compositionMode must be one of single, any, all".to_owned(),
        )),
    }
}

fn normalize_group_type(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "api_group" => Ok("api_group".to_owned()),
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "groupType must be api_group".to_owned(),
        )),
    }
}

fn normalize_selection_mode(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "manual" | "all" | "any" | "dynamic_all_api" => Ok(value.trim().to_ascii_lowercase()),
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "selectionMode must be one of manual, all, any, dynamic_all_api".to_owned(),
        )),
    }
}

fn normalize_status(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "active" | "disabled" | "inactive" => Ok(value.trim().to_ascii_lowercase()),
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "status must be one of active, disabled, inactive".to_owned(),
        )),
    }
}

fn normalize_member_role(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "included" | "optional" | "fallback" => Ok(value.trim().to_ascii_lowercase()),
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "memberRole must be one of included, optional, fallback".to_owned(),
        )),
    }
}

fn normalize_group_item_role(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "included" | "optional" | "fallback" => Ok(value.trim().to_ascii_lowercase()),
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "itemRole must be one of included, optional, fallback".to_owned(),
        )),
    }
}

fn is_dynamic_all_group(_group_code: &str, selection_mode: &str) -> bool {
    selection_mode == "dynamic_all_api"
}

fn optional_non_negative(
    value: Option<&Value>,
    field_name: &str,
) -> Result<Option<i64>, AiResourceCommandBuildError> {
    let value = parse_optional_i64_value(value, field_name)?;
    if value.is_some_and(|value| value < 0) {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "{field_name} must be a non-negative integer"
        )));
    }
    Ok(value)
}

fn parse_optional_i64_value(
    value: Option<&Value>,
    field_name: &str,
) -> Result<Option<i64>, AiResourceCommandBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let raw = match value {
        Value::String(value) => value.trim().to_owned(),
        Value::Number(value) => value.to_string(),
        _ => {
            return Err(AiResourceCommandBuildError::BadRequest(format!(
                "{field_name} must be an integer string"
            )))
        }
    };
    if raw.is_empty() {
        return Ok(None);
    }
    raw.parse::<i64>().map(Some).map_err(|_| {
        AiResourceCommandBuildError::BadRequest(format!("{field_name} must be an integer string"))
    })
}

fn parse_positive_id(value: &str, field_name: &str) -> Result<i64, String> {
    let id = value
        .trim()
        .parse::<i64>()
        .map_err(|_| format!("{field_name} must be a positive integer"))?;
    if id <= 0 {
        return Err(format!("{field_name} must be a positive integer"));
    }
    Ok(id)
}

fn generate_entity_uuid(
    state: &AdminAiResourceState,
) -> Result<String, AiResourceCommandBuildError> {
    state
        .entity_uuid_generator
        .generate_entity_uuid()
        .map_err(AiResourceCommandBuildError::System)
}

fn generate_entity_uuids(
    state: &AdminAiResourceState,
    count: usize,
) -> Result<Vec<String>, AiResourceCommandBuildError> {
    (0..count).map(|_| generate_entity_uuid(state)).collect()
}

fn request_id_error(error: RequestIdError) -> AiResourceCommandBuildError {
    match error {
        RequestIdError::Invalid(message) => AiResourceCommandBuildError::BadRequest(message),
        RequestIdError::System(message) => {
            AiResourceCommandBuildError::System(DomainError::new(message))
        }
    }
}

fn command_build_error_response(
    ctx: &WebRequestContext,
    error: AiResourceCommandBuildError,
) -> Response {
    match error {
        AiResourceCommandBuildError::BadRequest(message) => bad_request(&ctx, message),
        AiResourceCommandBuildError::System(error) => {
            ai_resource_system_response(&ctx, "AI resource command is invalid", error)
        }
    }
}

fn bad_request(ctx: &WebRequestContext, message: String) -> Response {
    problem_for(ctx, SdkWorkResultCode::ValidationError, message)
}

fn not_found_response(ctx: &WebRequestContext, message: impl Into<String>) -> Response {
    problem_for(ctx, SdkWorkResultCode::NotFound, message.into())
}

fn conflict_response(ctx: &WebRequestContext, error: DomainError) -> Response {
    problem_for(ctx, SdkWorkResultCode::Conflict, error.to_string())
}

fn current_timestamp_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_unix_timestamp(seconds)
}

fn format_unix_timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

fn ai_resource_system_response(
    ctx: &WebRequestContext,
    context: &str,
    error: DomainError,
) -> Response {
    problem_for(
        ctx,
        SdkWorkResultCode::InternalError,
        format!("{context}: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_non_negative_accepts_int64_string_values() {
        let value = Value::String("42".to_owned());

        let parsed = match optional_non_negative(Some(&value), "sortOrder") {
            Ok(parsed) => parsed,
            Err(_) => panic!("string encoded int64 sort order should be accepted"),
        };

        assert_eq!(parsed, Some(42));
    }

    #[test]
    fn optional_non_negative_treats_null_as_clear_value() {
        let parsed = match optional_non_negative(Some(&Value::Null), "sortOrder") {
            Ok(parsed) => parsed,
            Err(_) => panic!("null sort order should be accepted as an explicit clear value"),
        };

        assert_eq!(parsed, None);
    }

    #[test]
    fn optional_non_negative_rejects_negative_int64_strings() {
        let value = Value::String("-1".to_owned());
        let error = optional_non_negative(Some(&value), "sortOrder")
            .expect_err("negative sort order must be rejected");

        match error {
            AiResourceCommandBuildError::BadRequest(message) => {
                assert!(message.contains("non-negative integer"));
            }
            AiResourceCommandBuildError::System(_) => {
                panic!("negative sort order should be a validation error");
            }
        }
    }

    #[test]
    fn validate_page_query_rejects_offset_overflow() {
        let error = validate_page_query(Some(i64::MAX), Some(MAX_LIST_PAGE_SIZE))
            .expect_err("overflowing offset must be rejected");

        assert_eq!(error, "page is too large");
    }

    #[test]
    fn validate_page_query_applies_defaults_and_rejects_invalid_bounds() {
        assert_eq!(
            validate_page_query(None, None).expect("default page query should be valid"),
            (1, DEFAULT_LIST_PAGE_SIZE, 0)
        );
        assert_eq!(
            validate_page_query(Some(3), Some(25)).expect("bounded page query should be valid"),
            (3, 25, 50)
        );
        assert_eq!(
            validate_page_query(Some(0), Some(20)).expect_err("page zero must be rejected"),
            "page must be greater than or equal to 1"
        );
        for page_size in [0, MAX_LIST_PAGE_SIZE + 1] {
            assert_eq!(
                validate_page_query(Some(1), Some(page_size))
                    .expect_err("out-of-range page_size must be rejected"),
                "page_size must be between 1 and 200"
            );
        }
    }

    #[test]
    fn validate_list_search_query_trims_and_bounds_unicode_characters() {
        let accepted = validate_list_search_query(Some(format!("  {}  ", "模".repeat(256))))
            .expect("256 Unicode characters should be accepted");
        assert_eq!(accepted, Some("模".repeat(256)));

        let error = validate_list_search_query(Some("模".repeat(257)))
            .expect_err("search queries above 256 characters must be rejected");
        assert_eq!(error, "q must not exceed 256 characters");
        assert_eq!(
            validate_list_search_query(Some("   ".to_owned())).expect("blank search is valid"),
            None
        );
    }

    #[test]
    fn optional_text_counts_unicode_characters_instead_of_utf8_bytes() {
        let accepted = match optional_text(Some("模".repeat(128)), "groupName", 128) {
            Ok(value) => value,
            Err(_) => panic!("128 Unicode characters should be accepted"),
        };
        assert_eq!(accepted, Some("模".repeat(128)));
        assert!(optional_text(Some("模".repeat(129)), "groupName", 128).is_err());
    }

    #[test]
    fn resource_group_members_are_bounded_to_512_items() {
        let members = (0..=MAX_MEMBERS)
            .map(|index| AiResourceGroupMemberRequest {
                resource_code: Some(format!("resource.{index}")),
                item_role: None,
                sort_order: None,
            })
            .collect();
        let error = normalize_group_members(members)
            .expect_err("resource groups above the member limit must be rejected");
        match error {
            AiResourceCommandBuildError::BadRequest(message) => {
                assert_eq!(message, "members must contain at most 512 items");
            }
            AiResourceCommandBuildError::System(_) => {
                panic!("member limit must be a validation error");
            }
        }
    }

    #[test]
    fn member_update_rejects_unknown_json_fields() {
        let error = parse_json_body::<AiResourceGroupMemberUpdateRequest>(
            br#"{"itemRole":"included","legacyRole":"fallback"}"#,
            "AI resource group member update",
        )
        .expect_err("unknown request fields must be rejected");

        assert!(error.contains("unknown field `legacyRole`"));
    }

    #[test]
    fn model_access_channel_fields_are_strict_and_default_to_all_agent_providers() {
        assert_eq!(
            normalize_resource_type("MODEL_ACCESS_CHANNEL".to_owned())
                .expect("model access channel resource type"),
            "model_access_channel"
        );
        assert_eq!(
            normalize_access_channel_kind(Some(" Relay ".to_owned()), true, true)
                .expect("relay kind"),
            Some("relay".to_owned())
        );
        assert_eq!(
            normalize_supported_agent_provider_ids(None, true, true).expect("default providers"),
            canonical_agent_provider_ids()
        );
        let error = normalize_agent_provider_ids(vec!["unknown-agent".to_owned()], false)
            .expect_err("unknown provider must be rejected");
        match error {
            AiResourceCommandBuildError::BadRequest(message) => {
                assert!(message.contains("unsupported provider"));
            }
            AiResourceCommandBuildError::System(_) => panic!("expected validation error"),
        }
    }

    #[test]
    fn model_access_channel_base_url_requires_public_http_origin_without_credentials() {
        assert_eq!(
            normalize_base_url(Some("https://relay.example.test/v1".to_owned()), true, true,)
                .expect("valid relay URL"),
            Some("https://relay.example.test/v1".to_owned())
        );
        for invalid in [
            "relative/path",
            "ftp://relay.example.test/v1",
            "https://user:secret@relay.example.test/v1",
        ] {
            assert!(normalize_base_url(Some(invalid.to_owned()), true, true).is_err());
        }
    }

    #[test]
    fn ai_resource_requests_reject_api_key_fields() {
        let error = parse_json_body::<AiResourceCreateRequest>(
            br#"{
                "resourceCode":"channel.relay",
                "resourceType":"model_access_channel",
                "displayName":"Relay",
                "accessChannelKind":"relay",
                "baseUrl":"https://relay.example.test/v1",
                "apiKey":"must-never-enter-models"
            }"#,
            "AI resource",
        )
        .expect_err("API key fields must be rejected");
        assert!(error.contains("unknown field `apiKey`"));
    }
}
