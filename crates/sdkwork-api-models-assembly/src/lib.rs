//! Application API assembly for sdkwork-models.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM: preserve the authored assembly bootstrap and UUID adapter.

mod bootstrap;
mod contribution;
mod entity_uuid_generator;
mod generated;

pub use bootstrap::{assemble_api_router, ApiAssembly, ApiAssemblyContribution, assemble_business_routes, web_module};
pub use contribution::assemble_app_api_contribution;

/// Runs Models-owned database lifecycle before dependent assemblies load the
/// shared model catalog.
pub async fn bootstrap_database_from_env() -> Result<(), String> {
    sdkwork_models_database_host::bootstrap_models_database_from_env()
        .await
        .map(|_| ())
}

/// App-api surface route manifest owned by the dependency assembly.
pub fn app_api_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    sdkwork_routes_models_catalog_app_api::app_route_manifest()
}

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
