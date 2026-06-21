use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, put};
use axum::{Json, Router};
use sdkwork_claw_http::TrustedRequestSubject;
use serde::{Deserialize, Serialize};

use crate::api::request_id::{generate_server_request_id, RequestIdError};
use crate::api::response::PlusApiResult;
use crate::application::EntityUuidGenerator;
use crate::domain::DomainError;
use crate::ports::{
    AdminAiResourceGroupItem, AdminAiResourceGroupMemberCommand, AdminAiResourceGroupResourceItem,
    AdminAiResourceItem, AdminAiResourceMemberCommand, AdminAiResourceMemberItem,
    AdminAiResourceStore, AdminAiResourceSubject, CreateAdminAiResourceCommand,
    CreateAdminAiResourceGroupCommand, DeleteAdminAiResourceGroupCommand,
    ListAdminAiResourceGroupResourcesQuery, ListAdminAiResourceGroupsQuery,
    ListAdminAiResourcesQuery, UpdateAdminAiResourceCommand, UpdateAdminAiResourceGroupCommand,
};

const MAX_RESOURCE_CODE_LEN: usize = 192;
const MAX_RESOURCE_TYPE_LEN: usize = 64;
const MAX_DISPLAY_NAME_LEN: usize = 128;
const MAX_VENDOR_CODE_LEN: usize = 64;
const MAX_MODALITY_CODE_LEN: usize = 64;
const MAX_API_ENDPOINT_CODE_LEN: usize = 128;
const MAX_CATALOG_KEY_LEN: usize = 256;
const MAX_MODEL_LEN: usize = 128;
const MAX_PROVIDER_NATIVE_MODEL_LEN: usize = 256;
const MAX_COMPOSITION_MODE_LEN: usize = 32;
const MAX_GROUP_CODE_LEN: usize = 128;
const MAX_GROUP_NAME_LEN: usize = 128;
const MAX_GROUP_TYPE_LEN: usize = 64;
const MAX_SELECTION_MODE_LEN: usize = 32;
const MAX_DESCRIPTION_LEN: usize = 512;
const MAX_MEMBERS: usize = 512;

#[derive(Clone)]
struct AdminAiResourceState {
    store: Arc<dyn AdminAiResourceStore + Send + Sync>,
    entity_uuid_generator: Arc<dyn EntityUuidGenerator + Send + Sync>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourcesResponse {
    items: Vec<AdminAiResourceItemResponse>,
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
    capability: Option<String>,
    capabilities: Vec<String>,
    composition_mode: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<i64>,
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
    sort_order: Option<i64>,
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupItemEnvelope {
    item: AdminAiResourceGroupItemResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupDeleteResponse {
    deleted: bool,
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
    sort_order: Option<i64>,
    status: String,
    resource_count: i64,
    dynamic: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminAiResourceGroupResourcesResponse {
    items: Vec<AdminAiResourceGroupResourceItemResponse>,
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
    sort_order: Option<i64>,
    member_role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiResourceMemberRequest {
    member_resource_code: Option<String>,
    member_role: Option<String>,
    required: Option<bool>,
    sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    composition_mode: Option<String>,
    status: Option<String>,
    sort_order: Option<i64>,
    members: Option<Vec<AiResourceMemberRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    composition_mode: Option<String>,
    status: Option<String>,
    sort_order: Option<Option<i64>>,
    members: Option<Vec<AiResourceMemberRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiResourceGroupMemberRequest {
    resource_code: Option<String>,
    item_role: Option<String>,
    sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiResourceGroupCreateRequest {
    group_code: Option<String>,
    group_name: Option<String>,
    group_type: Option<String>,
    selection_mode: Option<String>,
    description: Option<String>,
    sort_order: Option<i64>,
    status: Option<String>,
    members: Option<Vec<AiResourceGroupMemberRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiResourceGroupUpdateRequest {
    group_code: Option<String>,
    group_name: Option<String>,
    group_type: Option<String>,
    selection_mode: Option<String>,
    description: Option<Option<String>>,
    sort_order: Option<Option<i64>>,
    status: Option<String>,
    members: Option<Vec<AiResourceGroupMemberRequest>>,
}

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
            "/backend/v3/api/ai/resource_groups/{group_id}",
            patch(update_ai_resource_group).delete(delete_ai_resource_group),
        )
        .with_state(AdminAiResourceState {
            store,
            entity_uuid_generator,
        })
}

async fn fetch_ai_resources(
    State(state): State<AdminAiResourceState>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
) -> Response {
    let subject = map_subject(trusted);

    match state
        .store
        .list_ai_resources(ListAdminAiResourcesQuery { subject })
        .await
    {
        Ok(items) => Json(PlusApiResult::success(AdminAiResourcesResponse {
            items: items.into_iter().map(to_item_response).collect(),
        }))
        .into_response(),
        Err(error) => ai_resource_system_response("AI resource read model is unavailable", error),
    }
}

async fn fetch_ai_resource_groups(
    State(state): State<AdminAiResourceState>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
) -> Response {
    let subject = map_subject(trusted);

    match state
        .store
        .list_ai_resource_groups(ListAdminAiResourceGroupsQuery { subject })
        .await
    {
        Ok(items) => Json(PlusApiResult::success(AdminAiResourceGroupsResponse {
            items: items.into_iter().map(to_group_response).collect(),
        }))
        .into_response(),
        Err(error) => {
            ai_resource_system_response("AI resource group read model is unavailable", error)
        }
    }
}

async fn fetch_ai_resource_group_resources(
    State(state): State<AdminAiResourceState>,
    Path(group_id_or_code): Path<String>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap
) -> Response {
    let subject = map_subject(trusted);
    let group_id_or_code = group_id_or_code.trim().to_owned();
    if group_id_or_code.is_empty() {
        return bad_request("AI resource group id or code is required".to_owned());
    }

    match state
        .store
        .list_ai_resource_group_resources(ListAdminAiResourceGroupResourcesQuery {
            subject,
            group_id_or_code,
        })
        .await
    {
        Ok(items) => Json(PlusApiResult::success(
            AdminAiResourceGroupResourcesResponse {
                items: items.into_iter().map(to_group_resource_response).collect(),
            },
        ))
        .into_response(),
        Err(error) if error.is_not_found() => not_found_response(error.to_string()),
        Err(error) => ai_resource_system_response(
            "AI resource group resource read model is unavailable",
            error,
        ),
    }
}

async fn create_ai_resource(
    State(state): State<AdminAiResourceState>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes
) -> Response {
    let subject = map_subject(trusted);
    let request = match parse_json_body::<AiResourceCreateRequest>(&body, "AI resource") {
        Ok(request) => request,
        Err(message) => return bad_request(message),
    };
    let command = match build_create_command(state.clone(), subject, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(error),
    };

    match state.store.create_ai_resource(command).await {
        Ok(item) => Json(PlusApiResult::success(AdminAiResourceItemEnvelope {
            item: to_item_response(item),
        }))
        .into_response(),
        Err(error) if error.is_not_found() => not_found_response(error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(error),
        Err(error) => {
            ai_resource_system_response("AI resource command store is unavailable", error)
        }
    }
}

async fn create_ai_resource_group(
    State(state): State<AdminAiResourceState>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes
) -> Response {
    let subject = map_subject(trusted);
    let request = match parse_json_body::<AiResourceGroupCreateRequest>(&body, "AI resource group")
    {
        Ok(request) => request,
        Err(message) => return bad_request(message),
    };
    let command = match build_group_create_command(state.clone(), subject, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(error),
    };

    match state.store.create_ai_resource_group(command).await {
        Ok(item) => Json(PlusApiResult::success(AdminAiResourceGroupItemEnvelope {
            item: to_group_response(item),
        }))
        .into_response(),
        Err(error) if error.is_not_found() => not_found_response(error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(error),
        Err(error) => {
            ai_resource_system_response("AI resource group command store is unavailable", error)
        }
    }
}

async fn update_ai_resource_group(
    State(state): State<AdminAiResourceState>,
    Path(group_id): Path<String>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes
) -> Response {
    let subject = map_subject(trusted);
    let group_id = match parse_positive_id(&group_id, "AI resource group id") {
        Ok(group_id) => group_id,
        Err(message) => return bad_request(message),
    };
    let request =
        match parse_json_body::<AiResourceGroupUpdateRequest>(&body, "AI resource group update") {
            Ok(request) => request,
            Err(message) => return bad_request(message),
        };
    let command = match build_group_update_command(state.clone(), subject, group_id, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(error),
    };

    match state.store.update_ai_resource_group(command).await {
        Ok(Some(item)) => Json(PlusApiResult::success(AdminAiResourceGroupItemEnvelope {
            item: to_group_response(item),
        }))
        .into_response(),
        Ok(None) => not_found_response("AI resource group was not found"),
        Err(error) if error.is_not_found() => not_found_response(error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(error),
        Err(error) => {
            ai_resource_system_response("AI resource group command store is unavailable", error)
        }
    }
}

async fn delete_ai_resource_group(
    State(state): State<AdminAiResourceState>,
    Path(group_id): Path<String>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap
) -> Response {
    let subject = map_subject(trusted);
    let group_id = match parse_positive_id(&group_id, "AI resource group id") {
        Ok(group_id) => group_id,
        Err(message) => return bad_request(message),
    };
    let command = match build_group_delete_command(state.clone(), subject, group_id) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(error),
    };

    match state.store.delete_ai_resource_group(command).await {
        Ok(deleted) => Json(PlusApiResult::success(AdminAiResourceGroupDeleteResponse {
            deleted,
        }))
        .into_response(),
        Err(error) if error.is_not_found() => not_found_response(error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(error),
        Err(error) => {
            ai_resource_system_response("AI resource group command store is unavailable", error)
        }
    }
}

async fn update_ai_resource(
    State(state): State<AdminAiResourceState>,
    Path(resource_id): Path<String>,
    trusted: TrustedRequestSubject,
    _headers: HeaderMap,
    body: Bytes
) -> Response {
    let subject = map_subject(trusted);
    let resource_id = match parse_positive_id(&resource_id, "AI resource id") {
        Ok(resource_id) => resource_id,
        Err(message) => return bad_request(message),
    };
    let request = match parse_json_body::<AiResourceUpdateRequest>(&body, "AI resource update") {
        Ok(request) => request,
        Err(message) => return bad_request(message),
    };
    let command = match build_update_command(state.clone(), subject, resource_id, request) {
        Ok(command) => command,
        Err(error) => return command_build_error_response(error),
    };

    match state.store.update_ai_resource(command).await {
        Ok(Some(item)) => Json(PlusApiResult::success(AdminAiResourceItemEnvelope {
            item: to_item_response(item),
        }))
        .into_response(),
        Ok(None) => not_found_response("AI resource was not found"),
        Err(error) if error.is_not_found() => not_found_response(error.to_string()),
        Err(error) if error.is_conflict() => conflict_response(error),
        Err(error) => {
            ai_resource_system_response("AI resource command store is unavailable", error)
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
        capability: item.capability,
        capabilities: item.capabilities,
        composition_mode: item.composition_mode,
        status: item.status,
        sort_order: item.sort_order,
        members: item.members.into_iter().map(to_member_response).collect(),
    }
}

fn to_member_response(member: AdminAiResourceMemberItem) -> AdminAiResourceMemberResponse {
    AdminAiResourceMemberResponse {
        parent_resource_code: member.parent_resource_code,
        member_resource_code: member.member_resource_code,
        member_role: member.member_role,
        required: member.required,
        sort_order: member.sort_order,
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
        sort_order: item.sort_order,
        status: item.status,
        resource_count: item.resource_count,
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
        sort_order: item.sort_order,
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
        sort_order: optional_non_negative(request.sort_order, "sortOrder")?,
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
            .map(|value| optional_non_negative(value, "sortOrder"))
            .transpose()?,
        status: request.status.map(normalize_status).transpose()?,
        members,
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
        resource_type: normalize_resource_type(required_text(
            request.resource_type,
            "resourceType",
            "AI resource type",
            MAX_RESOURCE_TYPE_LEN,
        )?)?,
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
        sort_order: optional_non_negative(request.sort_order, "sortOrder")?,
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
            sort_order: optional_non_negative(member.sort_order, "member.sortOrder")?,
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
    Ok(UpdateAdminAiResourceCommand {
        subject,
        resource_id,
        member_uuids: generate_entity_uuids(&state, member_count)?,
        audit_log_uuid: generate_entity_uuid(&state)?,
        resource_code: request
            .resource_code
            .map(|value| resource_code_value(value, "resourceCode", MAX_RESOURCE_CODE_LEN))
            .transpose()?,
        resource_type: request
            .resource_type
            .map(|value| {
                normalize_resource_type(required_text(
                    Some(value),
                    "resourceType",
                    "AI resource type",
                    MAX_RESOURCE_TYPE_LEN,
                )?)
            })
            .transpose()?,
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
        composition_mode: request
            .composition_mode
            .map(normalize_composition_mode)
            .transpose()?,
        status: request.status.map(normalize_status).transpose()?,
        sort_order: request
            .sort_order
            .map(|value| optional_non_negative(value, "sortOrder"))
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
            sort_order: optional_non_negative(member.sort_order, "member.sortOrder")?,
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
    if value.len() > max_len {
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

fn normalize_resource_type(value: String) -> Result<String, AiResourceCommandBuildError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "vendor" | "modality" | "api_endpoint" | "model_api" | "bundle" => {
            Ok(value.trim().to_ascii_lowercase())
        }
        _ => Err(AiResourceCommandBuildError::BadRequest(
            "resourceType must be one of vendor, modality, api_endpoint, model_api, bundle"
                .to_owned(),
        )),
    }
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
    value: Option<i64>,
    field_name: &str,
) -> Result<Option<i64>, AiResourceCommandBuildError> {
    if value.is_some_and(|value| value < 0) {
        return Err(AiResourceCommandBuildError::BadRequest(format!(
            "{field_name} must be a non-negative integer"
        )));
    }
    Ok(value)
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

fn command_build_error_response(error: AiResourceCommandBuildError) -> Response {
    match error {
        AiResourceCommandBuildError::BadRequest(message) => bad_request(message),
        AiResourceCommandBuildError::System(error) => {
            ai_resource_system_response("AI resource command is invalid", error)
        }
    }
}

fn bad_request(message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(PlusApiResult::error("4001", message)),
    )
        .into_response()
}

fn not_found_response(message: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(PlusApiResult::error("4040", message.into())),
    )
        .into_response()
}

fn conflict_response(error: DomainError) -> Response {
    (
        StatusCode::CONFLICT,
        Json(PlusApiResult::error("4090", error.to_string())),
    )
        .into_response()
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

fn ai_resource_system_response(context: &str, error: DomainError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(PlusApiResult::error("5000", format!("{context}: {error}"))),
    )
        .into_response()
}
