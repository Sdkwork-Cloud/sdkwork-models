use std::collections::HashSet;

use sdkwork_models_contract_service::{
    AdminAiResourceHierarchyNodeCommand, DomainError, DomainResult,
    ReplaceAdminAiResourceHierarchyCommand,
};

pub(crate) fn validate_hierarchy_command(
    command: &ReplaceAdminAiResourceHierarchyCommand,
) -> DomainResult<HashSet<String>> {
    if command.root_resource_code.trim().is_empty() || command.nodes.is_empty() {
        return Err(DomainError::new(
            "AI resource hierarchy requires a root code and at least one node",
        ));
    }
    let root_prefix = format!("{}.", command.root_resource_code);
    if command.owned_resource_code_prefixes.is_empty()
        || command.owned_resource_code_prefixes.iter().any(|prefix| {
            !prefix.starts_with(&root_prefix) || !prefix.ends_with('.') || prefix == &root_prefix
        })
    {
        return Err(DomainError::new(
            "AI resource hierarchy ownership prefixes must be descendants of the root code",
        ));
    }

    let mut resource_codes = HashSet::with_capacity(command.nodes.len());
    for node in &command.nodes {
        if node.resource_code != command.root_resource_code
            && !command
                .owned_resource_code_prefixes
                .iter()
                .any(|prefix| node.resource_code.starts_with(prefix))
        {
            return Err(DomainError::new(format!(
                "AI resource hierarchy node is outside the owned prefixes: {}",
                node.resource_code
            )));
        }
        if node.member_uuids.len() != node.members.len() {
            return Err(DomainError::new(format!(
                "AI resource hierarchy member UUID count does not match members for {}",
                node.resource_code
            )));
        }
        if !resource_codes.insert(node.resource_code.clone()) {
            return Err(DomainError::conflict(format!(
                "duplicate AI resource hierarchy node: {}",
                node.resource_code
            )));
        }
    }
    if !resource_codes.contains(&command.root_resource_code) {
        return Err(DomainError::new(
            "AI resource hierarchy root node is missing",
        ));
    }
    for node in &command.nodes {
        if let Some(member) = node
            .members
            .iter()
            .find(|member| !resource_codes.contains(&member.member_resource_code))
        {
            return Err(DomainError::new(format!(
                "AI resource hierarchy member is outside the replacement graph: {}",
                member.member_resource_code
            )));
        }
    }
    Ok(resource_codes)
}

pub(crate) fn hierarchy_node_schema(node: &AdminAiResourceHierarchyNodeCommand) -> String {
    let mut schema = serde_json::Map::new();
    schema.insert(
        "compositionMode".to_owned(),
        serde_json::Value::String(node.composition_mode.clone()),
    );
    insert_optional_string(&mut schema, "accessChannelKind", &node.access_channel_kind);
    insert_optional_string(&mut schema, "baseUrl", &node.base_url);
    insert_optional_string(&mut schema, "defaultVendorCode", &node.default_vendor_code);
    insert_optional_string(&mut schema, "defaultModelId", &node.default_model_id);
    if !node.supported_agent_provider_ids.is_empty() {
        schema.insert(
            "supportedAgentProviderIds".to_owned(),
            serde_json::json!(&node.supported_agent_provider_ids),
        );
    }
    insert_optional_i64(&mut schema, "contextTokens", node.context_tokens);
    insert_optional_i64(&mut schema, "maxOutputTokens", node.max_output_tokens);
    insert_optional_i64(&mut schema, "toolCallRounds", node.tool_call_rounds);
    if let Some(value) = node.supports_multimodal {
        schema.insert(
            "supportsMultimodal".to_owned(),
            serde_json::Value::Bool(value),
        );
    }
    insert_optional_string(&mut schema, "description", &node.description);
    serde_json::Value::Object(schema).to_string()
}

pub(crate) fn resource_code_is_owned(
    command: &ReplaceAdminAiResourceHierarchyCommand,
    resource_code: &str,
) -> bool {
    command
        .owned_resource_code_prefixes
        .iter()
        .any(|prefix| resource_code.starts_with(prefix))
}

fn insert_optional_string(
    schema: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &Option<String>,
) {
    if let Some(value) = value {
        schema.insert(key.to_owned(), serde_json::Value::String(value.clone()));
    }
}

fn insert_optional_i64(
    schema: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<i64>,
) {
    if let Some(value) = value {
        schema.insert(key.to_owned(), serde_json::Value::Number(value.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_models_contract_service::{AdminAiResourceMemberCommand, AdminAiResourceSubject};

    #[test]
    fn hierarchy_validation_rejects_external_and_duplicate_nodes() {
        let mut command = command_fixture();
        command.nodes.push(command.nodes[0].clone());
        assert!(validate_hierarchy_command(&command)
            .expect_err("duplicate node must fail")
            .is_conflict());

        let mut command = command_fixture();
        command.nodes[0].resource_code = "other.model.1".to_owned();
        assert!(validate_hierarchy_command(&command)
            .expect_err("external node must fail")
            .to_string()
            .contains("outside the owned prefixes"));
    }

    fn command_fixture() -> ReplaceAdminAiResourceHierarchyCommand {
        let model_code = "channel.relay.model.1.1".to_owned();
        ReplaceAdminAiResourceHierarchyCommand {
            subject: AdminAiResourceSubject {
                tenant_id: 7,
                organization_id: 9,
                operator_id: 11,
                operator_type: 1,
            },
            root_resource_code: "channel.relay".to_owned(),
            owned_resource_code_prefixes: vec![
                "channel.relay.vendor.".to_owned(),
                "channel.relay.model.".to_owned(),
            ],
            nodes: vec![
                AdminAiResourceHierarchyNodeCommand {
                    resource_uuid: "model-uuid".to_owned(),
                    member_uuids: Vec::new(),
                    resource_code: model_code.clone(),
                    resource_type: "model".to_owned(),
                    route_kind: None,
                    display_name: "Model".to_owned(),
                    vendor_code: Some("openai".to_owned()),
                    modality_code: None,
                    api_endpoint_code: None,
                    catalog_key: Some("openai/model".to_owned()),
                    model: Some("model".to_owned()),
                    provider_native_model: Some("model".to_owned()),
                    access_channel_kind: None,
                    base_url: None,
                    default_vendor_code: None,
                    default_model_id: None,
                    supported_agent_provider_ids: Vec::new(),
                    context_tokens: Some(128_000),
                    max_output_tokens: Some(16_384),
                    tool_call_rounds: Some(32),
                    supports_multimodal: Some(true),
                    description: None,
                    composition_mode: "single".to_owned(),
                    status: "active".to_owned(),
                    sort_order: None,
                    members: Vec::new(),
                },
                AdminAiResourceHierarchyNodeCommand {
                    resource_uuid: "root-uuid".to_owned(),
                    member_uuids: vec!["member-uuid".to_owned()],
                    resource_code: "channel.relay".to_owned(),
                    resource_type: "model_access_channel".to_owned(),
                    route_kind: None,
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
                    default_model_id: Some("model".to_owned()),
                    supported_agent_provider_ids: vec!["codex".to_owned()],
                    context_tokens: None,
                    max_output_tokens: None,
                    tool_call_rounds: None,
                    supports_multimodal: None,
                    description: None,
                    composition_mode: "all".to_owned(),
                    status: "active".to_owned(),
                    sort_order: None,
                    members: vec![AdminAiResourceMemberCommand {
                        member_resource_code: model_code,
                        member_role: "model".to_owned(),
                        required: true,
                        sort_order: Some(0),
                    }],
                },
            ],
            audit_log_uuid: "audit-uuid".to_owned(),
            request_id: "request-id".to_owned(),
            requested_at: "2026-08-02T00:00:00Z".to_owned(),
        }
    }
}
