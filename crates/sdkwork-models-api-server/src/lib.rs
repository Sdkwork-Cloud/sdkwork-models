//! Canonical HTTP route composition for the `sdkwork-models` intelligence catalog product.

mod catalog_app_router;
mod catalog_backend_router;
mod entity_uuid_generator;
mod health_router;
mod json_pricing_catalog;
mod service_host;

pub use catalog_app_router::{
    catalog_app_router_with_catalog, catalog_app_router_with_catalog_and_read_store,
    catalog_app_router_with_postgres_pool, catalog_app_router_with_sqlite_pool,
};
pub use catalog_backend_router::{
    catalog_backend_router_with_postgres_pool, catalog_backend_router_with_sqlite_pool,
    catalog_backend_router_with_stores, catalog_backend_router_without_stores,
};
pub use entity_uuid_generator::CatalogEntityUuidGenerator;
pub use health_router::{models_health_router, models_health_router_with_readiness, ModelsReadinessProbe};
pub use json_pricing_catalog::JsonPricingCatalog;
pub use service_host::ModelsServiceHost;
pub use sdkwork_models_catalog_service::PricingCatalog;
pub use sdkwork_router_models_catalog_backend_api::{
    backend_route_manifest, intelligence_catalog_backend_api_prefixes,
    intelligence_catalog_backend_api_public_path_prefixes,
    wrap_router_with_web_framework as wrap_backend_router_with_web_framework,
    wrap_router_with_web_framework_from_env as wrap_backend_router_with_web_framework_from_env,
};
pub use sdkwork_router_models_catalog_app_api::{
    app_route_manifest, intelligence_catalog_app_api_prefixes,
    intelligence_catalog_app_api_public_path_prefixes,
    wrap_router_with_web_framework as wrap_app_router_with_web_framework,
    wrap_router_with_web_framework_from_env as wrap_app_router_with_web_framework_from_env,
};

pub const APP_API_PREFIX: &str = "/app/v3/api";
pub const BACKEND_API_PREFIX: &str = "/backend/v3/api";

pub fn compose_models_health_and_backend_router_with_sqlite_pool(pool: sqlx::SqlitePool) -> axum::Router {
    models_health_router_with_readiness(ModelsReadinessProbe::Sqlite(pool.clone()))
        .merge(catalog_backend_router_with_sqlite_pool(pool))
}

pub fn compose_models_health_and_backend_router_with_postgres_pool(pool: sqlx::PgPool) -> axum::Router {
    models_health_router_with_readiness(ModelsReadinessProbe::Postgres(pool.clone()))
        .merge(catalog_backend_router_with_postgres_pool(pool))
}
