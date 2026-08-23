use crate::domain::{DomainError, DomainResult};

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
    PrimaryAccount,
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
            6 => Ok(Self::PrimaryAccount),
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
            Self::PrimaryAccount => 6,
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
            | Self::PrimaryAccount
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
                "routing capability contains unsupported value: {value}"
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteCandidate {
    pub account_group_id: i64,
    pub weight: i64,
    pub region_code: Option<String>,
}

impl RouteCandidate {
    pub fn new(account_group_id: i64, weight: i64) -> Self {
        Self {
            account_group_id,
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

