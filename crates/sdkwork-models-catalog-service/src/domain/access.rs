use crate::domain::DecimalValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayApiKey {
    pub id: i64,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub user_id: i64,
    pub group_id: i64,
    pub name: String,
    pub key_prefix: String,
    pub key_display_masked: String,
    pub key_hash: String,
    pub copyable_key: Option<String>,
    pub policy_id: Option<i64>,
    pub quota_policy_id: Option<i64>,
    pub created_at: String,
    pub expire_at: Option<String>,
    pub status_code: i32,
    pub default_for_runtime: bool,
    pub group_bindings: Vec<GatewayApiKeyChannelGroupBinding>,
}

impl GatewayApiKey {
    pub fn new(id: i64, group_id: i64, key_prefix: &str, key_hash: &str) -> Self {
        let key_prefix = key_prefix.to_owned();
        Self {
            id,
            tenant_id: 0,
            organization_id: 0,
            user_id: 0,
            group_id,
            name: key_prefix.clone(),
            key_display_masked: mask_key_prefix(&key_prefix),
            key_prefix,
            key_hash: key_hash.to_owned(),
            copyable_key: None,
            policy_id: None,
            quota_policy_id: None,
            created_at: String::new(),
            expire_at: None,
            status_code: 1,
            default_for_runtime: false,
            group_bindings: Vec::new(),
        }
    }

    pub fn with_owner(mut self, tenant_id: i64, organization_id: i64, user_id: i64) -> Self {
        self.tenant_id = tenant_id;
        self.organization_id = organization_id;
        self.user_id = user_id;
        self
    }

    pub fn with_management_metadata(
        mut self,
        name: &str,
        key_display_masked: &str,
        policy_id: Option<i64>,
        quota_policy_id: Option<i64>,
        created_at: &str,
        expire_at: Option<&str>,
    ) -> Self {
        self.name = name.to_owned();
        self.key_display_masked = key_display_masked.to_owned();
        self.policy_id = policy_id;
        self.quota_policy_id = quota_policy_id;
        self.created_at = created_at.to_owned();
        self.expire_at = expire_at.map(str::to_owned);
        self
    }

    pub fn with_copyable_key(mut self, copyable_key: impl Into<String>) -> Self {
        self.copyable_key = Some(copyable_key.into());
        self
    }

    pub fn with_default_for_runtime(mut self, default_for_runtime: bool) -> Self {
        self.default_for_runtime = default_for_runtime;
        self
    }

    pub fn with_group_bindings(
        mut self,
        group_bindings: Vec<GatewayApiKeyChannelGroupBinding>,
    ) -> Self {
        self.group_bindings = normalized_group_bindings(group_bindings);
        self
    }

    pub fn effective_group_bindings(&self) -> Vec<GatewayApiKeyChannelGroupBinding> {
        if self.group_bindings.is_empty() {
            vec![GatewayApiKeyChannelGroupBinding::default_route(
                self.group_id,
            )]
        } else {
            self.group_bindings.clone()
        }
    }

    pub fn display_name(&self) -> String {
        if !self.name.trim().is_empty() {
            self.name.clone()
        } else {
            format!("API Key #{}", self.id)
        }
    }

    pub fn masked_key(&self) -> String {
        if self.key_display_masked.trim().is_empty() {
            mask_key_prefix(&self.key_prefix)
        } else {
            self.key_display_masked.clone()
        }
    }

    pub fn status_label(&self) -> &'static str {
        match self.status_code {
            1 => "enabled",
            _ => "disabled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayApiKeyChannelGroupBinding {
    pub group_id: i64,
    pub group_code: String,
    pub pricing_plan_code: String,
    pub binding_role: String,
    pub routing_strategy: String,
    pub priority: i32,
    pub weight: i32,
}

impl GatewayApiKeyChannelGroupBinding {
    pub fn new(
        group_id: i64,
        group_code: &str,
        pricing_plan_code: &str,
        priority: i32,
        weight: i32,
    ) -> Self {
        Self {
            group_id,
            group_code: group_code.trim().to_owned(),
            pricing_plan_code: pricing_plan_code.trim().to_owned(),
            binding_role: "route".to_owned(),
            routing_strategy: "auto".to_owned(),
            priority,
            weight,
        }
    }

    pub fn default_route(group_id: i64) -> Self {
        Self {
            group_id,
            group_code: String::new(),
            pricing_plan_code: String::new(),
            binding_role: "route".to_owned(),
            routing_strategy: "auto".to_owned(),
            priority: 100,
            weight: 100,
        }
    }

    pub fn with_binding_role(mut self, binding_role: &str) -> Self {
        self.binding_role = normalized_text_or(binding_role, "route");
        self
    }

    pub fn with_routing_strategy(mut self, routing_strategy: &str) -> Self {
        self.routing_strategy = normalized_text_or(routing_strategy, "auto");
        self
    }

    pub fn with_group_code(mut self, group_code: &str) -> Self {
        self.group_code = group_code.trim().to_owned();
        self
    }

    pub fn with_pricing_plan_code(mut self, pricing_plan_code: &str) -> Self {
        self.pricing_plan_code = pricing_plan_code.trim().to_owned();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelGroup {
    pub id: i64,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub name: String,
    pub code: String,
    pub pricing_plan_code: String,
    pub rate_multiplier: DecimalValue,
    pub official_price_multiplier: DecimalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayAccessPolicy {
    pub id: i64,
    pub allowed_capabilities: Vec<String>,
    pub ip_allowlist: Vec<String>,
}

impl GatewayAccessPolicy {
    pub fn new(id: i64, allowed_capabilities: Vec<String>, ip_allowlist: Vec<String>) -> Self {
        Self {
            id,
            allowed_capabilities,
            ip_allowlist,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelGroupMetricSnapshot {
    pub group_id: i64,
    pub capacity_used: Option<DecimalValue>,
    pub capacity_limit: Option<DecimalValue>,
    pub usage_amount_total: Option<DecimalValue>,
    pub snapshot_at: Option<String>,
}

impl ChannelGroupMetricSnapshot {
    pub fn new(
        group_id: i64,
        capacity_used: Option<DecimalValue>,
        capacity_limit: Option<DecimalValue>,
        usage_amount_total: Option<DecimalValue>,
        snapshot_at: Option<String>,
    ) -> Self {
        Self {
            group_id,
            capacity_used,
            capacity_limit,
            usage_amount_total,
            snapshot_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaPolicy {
    pub id: i64,
    pub quota_limit: Option<DecimalValue>,
    pub requests_per_second: Option<i64>,
    pub requests_per_day: Option<i64>,
    pub burst_limit: Option<DecimalValue>,
}

impl QuotaPolicy {
    pub fn new(id: i64, quota_limit: Option<DecimalValue>) -> Self {
        Self {
            id,
            quota_limit,
            requests_per_second: None,
            requests_per_day: None,
            burst_limit: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRiskRule {
    pub id: i64,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub rule_category: i32,
    pub rule_type: i32,
    pub scope_type: Option<i32>,
    pub scope_id: Option<i64>,
    pub target_type: i32,
    pub target_value: String,
    pub match_mode: i32,
    pub action: i32,
    pub priority: i32,
    pub requests_per_second: Option<i64>,
    pub requests_per_minute: Option<i64>,
    pub requests_per_day: Option<i64>,
    pub burst_limit: Option<DecimalValue>,
    pub block_duration_seconds: Option<i64>,
}

fn mask_key_prefix(key_prefix: &str) -> String {
    let key_prefix = key_prefix.trim();
    if key_prefix.is_empty() {
        "********".to_owned()
    } else {
        format!("{key_prefix}********")
    }
}

fn normalized_group_bindings(
    group_bindings: Vec<GatewayApiKeyChannelGroupBinding>,
) -> Vec<GatewayApiKeyChannelGroupBinding> {
    let mut bindings = group_bindings
        .into_iter()
        .filter(|binding| binding.group_id > 0)
        .map(|mut binding| {
            binding.binding_role = normalized_text_or(&binding.binding_role, "route");
            binding.routing_strategy = normalized_text_or(&binding.routing_strategy, "auto");
            binding
        })
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| {
        (
            binding.priority,
            std::cmp::Reverse(binding.weight),
            binding.group_id,
        )
    });
    bindings.dedup_by_key(|binding| binding.group_id);
    bindings
}

fn normalized_text_or(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

impl ChannelGroup {
    pub fn new(
        id: i64,
        code: &str,
        pricing_plan_code: &str,
        rate_multiplier: DecimalValue,
        official_price_multiplier: DecimalValue,
    ) -> Self {
        Self::new_scoped(
            id,
            0,
            0,
            code,
            pricing_plan_code,
            rate_multiplier,
            official_price_multiplier,
        )
    }

    pub fn new_scoped(
        id: i64,
        tenant_id: i64,
        organization_id: i64,
        code: &str,
        pricing_plan_code: &str,
        rate_multiplier: DecimalValue,
        official_price_multiplier: DecimalValue,
    ) -> Self {
        Self {
            id,
            tenant_id,
            organization_id,
            name: code.to_owned(),
            code: code.to_owned(),
            pricing_plan_code: pricing_plan_code.to_owned(),
            rate_multiplier,
            official_price_multiplier,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        let normalized = name.trim();
        if !normalized.is_empty() {
            self.name = normalized.to_owned();
        }
        self
    }

    pub fn display_name(&self) -> String {
        let normalized = self.name.trim();
        if normalized.is_empty() {
            self.code.clone()
        } else {
            normalized.to_owned()
        }
    }
}
