use crate::DomainResult;

pub trait EntityUuidGenerator {
    fn generate_entity_uuid(&self) -> DomainResult<String>;
}

pub trait ApiKeySecretGenerator: EntityUuidGenerator {
    fn generate_api_key_secret(&self) -> DomainResult<String>;
}
