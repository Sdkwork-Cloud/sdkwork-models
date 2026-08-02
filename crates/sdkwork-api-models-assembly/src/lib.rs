//! Application API assembly for sdkwork-models.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM: preserve the authored assembly bootstrap and UUID adapter.

mod bootstrap;
mod contribution;
mod entity_uuid_generator;
mod generated;

pub use bootstrap::{assemble_api_router, assemble_business_routes, ApiAssembly};
pub use contribution::assemble_app_api_contribution;

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
