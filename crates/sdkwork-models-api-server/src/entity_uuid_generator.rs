use sdkwork_id_core::uuid_v4;
use sdkwork_models_contract_service::{DomainResult, EntityUuidGenerator};

#[derive(Debug, Default, Clone, Copy)]
pub struct CatalogEntityUuidGenerator;

impl EntityUuidGenerator for CatalogEntityUuidGenerator {
    fn generate_entity_uuid(&self) -> DomainResult<String> {
        Ok(uuid_v4())
    }
}

impl CatalogEntityUuidGenerator {
    pub fn arc() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self)
    }
}
