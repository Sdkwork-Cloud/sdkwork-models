use serde_json::Value;

use crate::domain::{DomainError, DomainResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoutingPolicyScope {
    Global,
    Tenant,
    Organization,
    ApiKey,
    ChannelGroup,
}

impl RoutingPolicyScope {
    pub fn from_code(value: i32) -> DomainResult<Self> {
        match value {
            1 => Ok(Self::Global),
            2 => Ok(Self::Tenant),
            3 => Ok(Self::Organization),
            4 => Ok(Self::ApiKey),
            5 => Ok(Self::ChannelGroup),
            value => Err(DomainError::new(format!(
                "ai_routing_policy.policy_scope contains unsupported value: {value}"
            ))),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::Global => 1,
            Self::Tenant => 2,
            Self::Organization => 3,
            Self::ApiKey => 4,
            Self::ChannelGroup => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingCapability {
    Chat,
    Image,
    Audio,
    Music,
    Video,
    Embedding,
    Rerank,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiRouteStrategy {
    StatelessFailover,
    StatelessFailClosed,
    CreateThenSticky,
    ParentSticky,
    LookupSticky,
    PrimaryChannel,
    FanoutAggregate,
}

impl AiRouteStrategy {
    pub fn from_code(value: i32) -> DomainResult<Self> {
        match value {
            1 => Ok(Self::StatelessFailover),
            2 => Ok(Self::StatelessFailClosed),
            3 => Ok(Self::CreateThenSticky),
            4 => Ok(Self::ParentSticky),
            5 => Ok(Self::LookupSticky),
            6 => Ok(Self::PrimaryChannel),
            7 => Ok(Self::FanoutAggregate),
            value => Err(DomainError::new(format!(
                "ai_route_taxonomy.route_strategy contains unsupported value: {value}"
            ))),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::StatelessFailover => 1,
            Self::StatelessFailClosed => 2,
            Self::CreateThenSticky => 3,
            Self::ParentSticky => 4,
            Self::LookupSticky => 5,
            Self::PrimaryChannel => 6,
            Self::FanoutAggregate => 7,
        }
    }

    pub fn failure_strategy(self) -> AiRouteFailureStrategy {
        match self {
            Self::StatelessFailover => AiRouteFailureStrategy::Failover,
            Self::StatelessFailClosed
            | Self::CreateThenSticky
            | Self::ParentSticky
            | Self::LookupSticky
            | Self::PrimaryChannel
            | Self::FanoutAggregate => AiRouteFailureStrategy::FailClosed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiRouteFailureStrategy {
    Failover,
    FailClosed,
}

impl AiRouteFailureStrategy {
    pub fn from_code(value: i32) -> DomainResult<Self> {
        match value {
            1 => Ok(Self::Failover),
            2 => Ok(Self::FailClosed),
            value => Err(DomainError::new(format!(
                "ai_route_taxonomy.failure_strategy contains unsupported value: {value}"
            ))),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::Failover => 1,
            Self::FailClosed => 2,
        }
    }

    pub fn should_try_next_route(self, is_last_route: bool) -> bool {
        matches!(self, Self::Failover) && !is_last_route
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiRouteModelRequirement {
    Required,
    Optional,
    Ignored,
}

impl AiRouteModelRequirement {
    pub fn from_code(value: i32) -> DomainResult<Self> {
        match value {
            1 => Ok(Self::Required),
            2 => Ok(Self::Optional),
            3 => Ok(Self::Ignored),
            value => Err(DomainError::new(format!(
                "ai_route_taxonomy.model_requirement contains unsupported value: {value}"
            ))),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::Required => 1,
            Self::Optional => 2,
            Self::Ignored => 3,
        }
    }

    pub fn routes_model_when_present(self) -> bool {
        matches!(self, Self::Required | Self::Optional)
    }

    pub fn permits_missing_model(self) -> bool {
        matches!(self, Self::Optional | Self::Ignored)
    }
}

impl RoutingCapability {
    pub fn from_code(value: i32) -> DomainResult<Self> {
        match value {
            1 => Ok(Self::Chat),
            2 => Ok(Self::Image),
            3 => Ok(Self::Audio),
            4 => Ok(Self::Music),
            5 => Ok(Self::Video),
            6 => Ok(Self::Embedding),
            7 => Ok(Self::Rerank),
            10 => Ok(Self::Network),
            value => Err(DomainError::new(format!(
                "ai_routing_policy.capability contains unsupported value: {value}"
            ))),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::Chat => 1,
            Self::Image => 2,
            Self::Audio => 3,
            Self::Music => 4,
            Self::Video => 5,
            Self::Embedding => 6,
            Self::Rerank => 7,
            Self::Network => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingFallbackMode {
    None,
    NextProvider,
    NextRegion,
    Cheapest,
    Fastest,
}

impl RoutingFallbackMode {
    pub fn from_code(value: i32) -> DomainResult<Self> {
        match value {
            1 => Ok(Self::None),
            2 => Ok(Self::NextProvider),
            3 => Ok(Self::NextRegion),
            4 => Ok(Self::Cheapest),
            5 => Ok(Self::Fastest),
            value => Err(DomainError::new(format!(
                "ai_routing_policy.fallback_mode contains unsupported value: {value}"
            ))),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::None => 1,
            Self::NextProvider => 2,
            Self::NextRegion => 3,
            Self::Cheapest => 4,
            Self::Fastest => 5,
        }
    }

    pub fn allows_rule_fallback_chain(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingPolicy {
    pub id: i64,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub policy_code: String,
    pub policy_scope: RoutingPolicyScope,
    pub subject_id: Option<i64>,
    pub capability: Option<RoutingCapability>,
    pub default_profile_id: Option<i64>,
    pub fallback_mode: Option<RoutingFallbackMode>,
}

impl RoutingPolicy {
    pub fn new(
        id: i64,
        tenant_id: i64,
        organization_id: i64,
        policy_code: &str,
        policy_scope: RoutingPolicyScope,
        subject_id: Option<i64>,
        default_profile_id: Option<i64>,
    ) -> Self {
        Self {
            id,
            tenant_id,
            organization_id,
            policy_code: policy_code.to_owned(),
            policy_scope,
            subject_id,
            capability: None,
            default_profile_id,
            fallback_mode: None,
        }
    }

    pub fn with_capability(mut self, capability: RoutingCapability) -> Self {
        self.capability = Some(capability);
        self
    }

    pub fn with_fallback_mode(mut self, fallback_mode: RoutingFallbackMode) -> Self {
        self.fallback_mode = Some(fallback_mode);
        self
    }

    pub fn fallback_mode_or_default(&self) -> RoutingFallbackMode {
        self.fallback_mode
            .unwrap_or(RoutingFallbackMode::NextProvider)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteCandidate {
    pub channel_id: i64,
    pub weight: i64,
    pub region_code: Option<String>,
}

impl RouteCandidate {
    pub fn new(channel_id: i64, weight: i64) -> Self {
        Self {
            channel_id,
            weight,
            region_code: None,
        }
    }

    pub fn with_region_code(mut self, region_code: &str) -> Self {
        self.region_code = normalized_optional_route_region_code(region_code);
        self
    }
}

fn normalized_optional_route_region_code(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingRule {
    pub id: i64,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub profile_id: i64,
    pub rule_code: String,
    pub priority: i32,
    pub match_expression: Value,
    pub target_model: Option<String>,
    pub candidate_channels: Vec<RouteCandidate>,
    pub fallback_chain: Vec<RouteCandidate>,
    pub constraints: Value,
}

impl RoutingRule {
    pub fn new(
        id: i64,
        tenant_id: i64,
        organization_id: i64,
        profile_id: i64,
        rule_code: &str,
        priority: i32,
        match_expression_json: &str,
        target_model: &str,
    ) -> Self {
        Self {
            id,
            tenant_id,
            organization_id,
            profile_id,
            rule_code: rule_code.to_owned(),
            priority,
            match_expression: serde_json::from_str(match_expression_json).unwrap_or(Value::Null),
            target_model: if target_model.trim().is_empty() {
                None
            } else {
                Some(target_model.to_owned())
            },
            candidate_channels: Vec::new(),
            fallback_chain: Vec::new(),
            constraints: Value::Object(Default::default()),
        }
    }

    pub fn with_candidate_channels(mut self, candidate_channels: Vec<RouteCandidate>) -> Self {
        self.candidate_channels = candidate_channels;
        self
    }

    pub fn with_fallback_chain(mut self, fallback_chain: Vec<RouteCandidate>) -> Self {
        self.fallback_chain = fallback_chain;
        self
    }

    pub fn matches_catalog_key(&self, catalog_key: &str, requested_model: &str) -> bool {
        if self.match_expression.is_null() {
            return true;
        }
        json_string_matches(&self.match_expression, "catalogKey", catalog_key)
            || json_string_matches(&self.match_expression, "catalog_key", catalog_key)
            || json_string_matches(&self.match_expression, "sourceModel", requested_model)
            || json_string_matches(&self.match_expression, "source_model", requested_model)
            || self
                .target_model
                .as_deref()
                .map(|target_model| target_model == catalog_key || target_model == requested_model)
                .unwrap_or(false)
    }

    pub fn matches_route_key(&self, route_key: &str) -> bool {
        if self.match_expression.is_null() {
            return true;
        }
        json_string_matches(&self.match_expression, "routeKey", route_key)
            || json_string_matches(&self.match_expression, "route_key", route_key)
            || json_string_matches(&self.match_expression, "catalogKey", route_key)
            || json_string_matches(&self.match_expression, "catalog_key", route_key)
            || self
                .target_model
                .as_deref()
                .map(|target_model| target_model == route_key)
                .unwrap_or(false)
    }
}

fn json_string_matches(value: &Value, field: &str, expected: &str) -> bool {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .map(|actual| actual == "*" || actual == expected)
        .unwrap_or(false)
}
