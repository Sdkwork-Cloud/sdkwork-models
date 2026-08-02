use std::future::Future;
use std::pin::Pin;

use crate::DomainResult;

pub type AdminAiResourceReadFuture<'a, T> =
    Pin<Box<dyn Future<Output = DomainResult<T>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdminAiResourceSubject {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub operator_id: i64,
    pub operator_type: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceMemberItem {
    pub parent_resource_code: String,
    pub member_resource_code: String,
    pub member_role: String,
    pub required: bool,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceItem {
    pub id: i64,
    pub resource_code: String,
    pub resource_type: String,
    pub display_name: String,
    pub vendor_code: Option<String>,
    pub modality_code: Option<String>,
    pub api_endpoint_code: Option<String>,
    pub catalog_key: Option<String>,
    pub model: Option<String>,
    pub provider_native_model: Option<String>,
    pub access_channel_kind: Option<String>,
    pub base_url: Option<String>,
    pub default_vendor_code: Option<String>,
    pub default_model_id: Option<String>,
    pub supported_agent_provider_ids: Vec<String>,
    pub context_tokens: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub tool_call_rounds: Option<i64>,
    pub supports_multimodal: Option<bool>,
    pub description: Option<String>,
    pub capability: Option<String>,
    pub capabilities: Vec<String>,
    pub composition_mode: String,
    pub status: String,
    pub sort_order: Option<i64>,
    pub members: Vec<AdminAiResourceMemberItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceGroupItem {
    pub id: i64,
    pub group_code: String,
    pub group_name: String,
    pub group_type: String,
    pub selection_mode: String,
    pub description: Option<String>,
    pub vendor_codes: Vec<String>,
    pub capability: Option<String>,
    pub capabilities: Vec<String>,
    pub sort_order: Option<i64>,
    pub status: String,
    pub resource_count: i64,
    pub dynamic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceGroupResourceItem {
    pub id: i64,
    pub resource_code: String,
    pub resource_type: String,
    pub display_name: String,
    pub vendor_code: Option<String>,
    pub modality_code: Option<String>,
    pub api_endpoint_code: Option<String>,
    pub catalog_key: Option<String>,
    pub model: Option<String>,
    pub provider_native_model: Option<String>,
    pub status: String,
    pub sort_order: Option<i64>,
    pub member_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAdminAiResourcesQuery {
    pub subject: AdminAiResourceSubject,
    pub q: Option<String>,
    pub resource_type: Option<String>,
    pub status: Option<String>,
    pub access_channel_kind: Option<String>,
    pub vendor_code: Option<String>,
    pub agent_provider_id: Option<String>,
    pub require_valid_access_channel_metadata: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAdminAiResourceGroupsQuery {
    pub subject: AdminAiResourceSubject,
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAdminAiResourceGroupResourcesQuery {
    pub subject: AdminAiResourceSubject,
    pub group_id_or_code: String,
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceListPage {
    pub items: Vec<AdminAiResourceItem>,
    pub total_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceGroupListPage {
    pub items: Vec<AdminAiResourceGroupItem>,
    pub total_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceGroupResourcesPage {
    pub items: Vec<AdminAiResourceGroupResourceItem>,
    pub total_count: i64,
}

impl ListAdminAiResourcesQuery {
    pub const DEFAULT_LIMIT: i64 = 20;
    pub const MAX_LIMIT: i64 = 200;

    pub fn normalized_limit(&self) -> i64 {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .clamp(1, Self::MAX_LIMIT)
    }

    pub fn normalized_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

impl ListAdminAiResourceGroupsQuery {
    pub const DEFAULT_LIMIT: i64 = 20;
    pub const MAX_LIMIT: i64 = 200;

    pub fn normalized_limit(&self) -> i64 {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .clamp(1, Self::MAX_LIMIT)
    }

    pub fn normalized_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

impl ListAdminAiResourceGroupResourcesQuery {
    pub const DEFAULT_LIMIT: i64 = 20;
    pub const MAX_LIMIT: i64 = 200;

    pub fn normalized_limit(&self) -> i64 {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .clamp(1, Self::MAX_LIMIT)
    }

    pub fn normalized_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceGroupMemberCommand {
    pub resource_code: String,
    pub item_role: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAdminAiResourceGroupCommand {
    pub subject: AdminAiResourceSubject,
    pub group_uuid: String,
    pub member_uuids: Vec<String>,
    pub audit_log_uuid: String,
    pub group_code: String,
    pub group_name: String,
    pub group_type: String,
    pub selection_mode: String,
    pub description: Option<String>,
    pub sort_order: Option<i64>,
    pub status: String,
    pub members: Vec<AdminAiResourceGroupMemberCommand>,
    pub request_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAdminAiResourceGroupCommand {
    pub subject: AdminAiResourceSubject,
    pub group_id: i64,
    pub member_uuids: Vec<String>,
    pub audit_log_uuid: String,
    pub group_code: Option<String>,
    pub group_name: Option<String>,
    pub group_type: Option<String>,
    pub selection_mode: Option<String>,
    pub description: Option<Option<String>>,
    pub sort_order: Option<Option<i64>>,
    pub status: Option<String>,
    pub members: Option<Vec<AdminAiResourceGroupMemberCommand>>,
    pub request_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertAdminAiResourceGroupMemberCommand {
    pub subject: AdminAiResourceSubject,
    pub group_id: i64,
    pub member_uuid: String,
    pub audit_log_uuid: String,
    pub member: AdminAiResourceGroupMemberCommand,
    pub request_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteAdminAiResourceGroupMemberCommand {
    pub subject: AdminAiResourceSubject,
    pub group_id: i64,
    pub resource_code: String,
    pub audit_log_uuid: String,
    pub request_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteAdminAiResourceGroupCommand {
    pub subject: AdminAiResourceSubject,
    pub group_id: i64,
    pub audit_log_uuid: String,
    pub request_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceMemberCommand {
    pub member_resource_code: String,
    pub member_role: String,
    pub required: bool,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAdminAiResourceCommand {
    pub subject: AdminAiResourceSubject,
    pub resource_uuid: String,
    pub member_uuids: Vec<String>,
    pub audit_log_uuid: String,
    pub resource_code: String,
    pub resource_type: String,
    pub display_name: String,
    pub vendor_code: Option<String>,
    pub modality_code: Option<String>,
    pub api_endpoint_code: Option<String>,
    pub catalog_key: Option<String>,
    pub model: Option<String>,
    pub provider_native_model: Option<String>,
    pub access_channel_kind: Option<String>,
    pub base_url: Option<String>,
    pub default_vendor_code: Option<String>,
    pub default_model_id: Option<String>,
    pub supported_agent_provider_ids: Vec<String>,
    pub description: Option<String>,
    pub composition_mode: String,
    pub status: String,
    pub sort_order: Option<i64>,
    pub members: Vec<AdminAiResourceMemberCommand>,
    pub request_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAdminAiResourceCommand {
    pub subject: AdminAiResourceSubject,
    pub resource_id: i64,
    pub member_uuids: Vec<String>,
    pub audit_log_uuid: String,
    pub resource_code: Option<String>,
    pub resource_type: Option<String>,
    pub display_name: Option<String>,
    pub vendor_code: Option<Option<String>>,
    pub modality_code: Option<Option<String>>,
    pub api_endpoint_code: Option<Option<String>>,
    pub catalog_key: Option<Option<String>>,
    pub model: Option<Option<String>>,
    pub provider_native_model: Option<Option<String>>,
    pub access_channel_kind: Option<String>,
    pub base_url: Option<String>,
    pub default_vendor_code: Option<String>,
    pub default_model_id: Option<String>,
    pub supported_agent_provider_ids: Option<Vec<String>>,
    pub description: Option<Option<String>>,
    pub composition_mode: Option<String>,
    pub status: Option<String>,
    pub sort_order: Option<Option<i64>>,
    pub members: Option<Vec<AdminAiResourceMemberCommand>>,
    pub request_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAiResourceHierarchyNodeCommand {
    pub resource_uuid: String,
    pub member_uuids: Vec<String>,
    pub resource_code: String,
    pub resource_type: String,
    pub display_name: String,
    pub vendor_code: Option<String>,
    pub modality_code: Option<String>,
    pub api_endpoint_code: Option<String>,
    pub catalog_key: Option<String>,
    pub model: Option<String>,
    pub provider_native_model: Option<String>,
    pub access_channel_kind: Option<String>,
    pub base_url: Option<String>,
    pub default_vendor_code: Option<String>,
    pub default_model_id: Option<String>,
    pub supported_agent_provider_ids: Vec<String>,
    pub context_tokens: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub tool_call_rounds: Option<i64>,
    pub supports_multimodal: Option<bool>,
    pub description: Option<String>,
    pub composition_mode: String,
    pub status: String,
    pub sort_order: Option<i64>,
    pub members: Vec<AdminAiResourceMemberCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceAdminAiResourceHierarchyCommand {
    pub subject: AdminAiResourceSubject,
    pub root_resource_code: String,
    pub owned_resource_code_prefixes: Vec<String>,
    pub nodes: Vec<AdminAiResourceHierarchyNodeCommand>,
    pub audit_log_uuid: String,
    pub request_id: String,
    pub requested_at: String,
}

pub trait AdminAiResourceStore {
    fn list_ai_resources<'a>(
        &'a self,
        query: ListAdminAiResourcesQuery,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceListPage>;

    fn create_ai_resource<'a>(
        &'a self,
        command: CreateAdminAiResourceCommand,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceItem>;

    fn update_ai_resource<'a>(
        &'a self,
        command: UpdateAdminAiResourceCommand,
    ) -> AdminAiResourceReadFuture<'a, Option<AdminAiResourceItem>>;

    fn replace_ai_resource_hierarchy<'a>(
        &'a self,
        command: ReplaceAdminAiResourceHierarchyCommand,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceItem>;

    fn list_ai_resource_groups<'a>(
        &'a self,
        query: ListAdminAiResourceGroupsQuery,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceGroupListPage>;

    fn list_ai_resource_group_resources<'a>(
        &'a self,
        query: ListAdminAiResourceGroupResourcesQuery,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceGroupResourcesPage>;

    fn create_ai_resource_group<'a>(
        &'a self,
        command: CreateAdminAiResourceGroupCommand,
    ) -> AdminAiResourceReadFuture<'a, AdminAiResourceGroupItem>;

    fn update_ai_resource_group<'a>(
        &'a self,
        command: UpdateAdminAiResourceGroupCommand,
    ) -> AdminAiResourceReadFuture<'a, Option<AdminAiResourceGroupItem>>;

    fn upsert_ai_resource_group_member<'a>(
        &'a self,
        command: UpsertAdminAiResourceGroupMemberCommand,
    ) -> AdminAiResourceReadFuture<'a, Option<AdminAiResourceGroupResourceItem>>;

    fn delete_ai_resource_group_member<'a>(
        &'a self,
        command: DeleteAdminAiResourceGroupMemberCommand,
    ) -> AdminAiResourceReadFuture<'a, bool>;

    fn delete_ai_resource_group<'a>(
        &'a self,
        command: DeleteAdminAiResourceGroupCommand,
    ) -> AdminAiResourceReadFuture<'a, bool>;
}
