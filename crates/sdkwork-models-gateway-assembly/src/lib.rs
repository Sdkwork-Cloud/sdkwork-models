//! Gateway assembly for sdkwork-models.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM: preserve application-specific service-host and router exports.

mod bootstrap;
mod catalog_app_router;
mod catalog_backend_router;
mod entity_uuid_generator;
mod generated;

pub use bootstrap::{
    assemble_application_business_router, assemble_application_router, ApplicationAssembly,
};
pub use catalog_app_router::{
    catalog_app_router_with_catalog, catalog_app_router_with_catalog_and_read_store,
    catalog_app_router_with_catalog_voice_and_read_store, catalog_app_router_with_postgres_pool,
    catalog_app_router_with_sqlite_pool,
};
pub use catalog_backend_router::{
    catalog_backend_router_with_postgres_pool,
    catalog_backend_router_with_postgres_pool_and_catalog, catalog_backend_router_with_sqlite_pool,
    catalog_backend_router_with_sqlite_pool_and_catalog, catalog_backend_router_with_stores,
    catalog_backend_router_without_stores,
};
pub use entity_uuid_generator::CatalogEntityUuidGenerator;
pub use sdkwork_models_catalog_service::PricingCatalog;
pub use sdkwork_models_service_host::{JsonPricingCatalog, ModelsServiceHost};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
