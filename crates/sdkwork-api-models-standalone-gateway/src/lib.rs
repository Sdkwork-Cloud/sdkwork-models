//! Canonical HTTP route composition for the `sdkwork-models` intelligence catalog product.

mod cors;
mod health_router;

pub use cors::application_cors_layer;
pub use health_router::{
    models_health_router, models_health_router_with_database_pool,
    models_health_router_with_readiness, ModelsReadinessProbe,
};
pub use sdkwork_api_models_assembly::{
    catalog_app_router_with_catalog, catalog_app_router_with_catalog_and_read_store,
    catalog_app_router_with_catalog_voice_and_read_store, catalog_app_router_with_postgres_pool,
    catalog_app_router_with_sqlite_pool, catalog_backend_router_with_postgres_pool,
    catalog_backend_router_with_postgres_pool_and_catalog, catalog_backend_router_with_sqlite_pool,
    catalog_backend_router_with_sqlite_pool_and_catalog, catalog_backend_router_with_stores,
    catalog_backend_router_without_stores, CatalogEntityUuidGenerator, JsonPricingCatalog,
    ModelsServiceHost, PricingCatalog,
};

pub const APP_API_PREFIX: &str = "/app/v3/api";
pub const BACKEND_API_PREFIX: &str = "/backend/v3/api";

pub fn compose_models_health_and_backend_router_with_sqlite_pool(
    pool: sqlx::SqlitePool,
) -> axum::Router {
    models_health_router_with_readiness(ModelsReadinessProbe::Sqlite(pool.clone()))
        .merge(catalog_backend_router_with_sqlite_pool(pool))
}

pub fn compose_models_health_and_backend_router_with_postgres_pool(
    pool: sqlx::PgPool,
) -> axum::Router {
    models_health_router_with_readiness(ModelsReadinessProbe::Postgres(pool.clone()))
        .merge(catalog_backend_router_with_postgres_pool(pool))
}
