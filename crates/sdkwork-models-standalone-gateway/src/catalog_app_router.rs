use std::sync::Arc;

use axum::Router;
use sdkwork_models_catalog_repository_sqlx::{
    PostgresModelRankingsReadStore, SqliteModelRankingsReadStore,
};
use sdkwork_models_catalog_service::{
    app_model_catalog_router, app_model_rankings_router, app_model_rankings_router_with_read_store,
    PricingCatalog,
};
use sdkwork_models_contract_service::ModelRankingsReadModelStore;
use sqlx::{PgPool, SqlitePool};

pub fn catalog_app_router_with_catalog<C>(catalog: Arc<C>) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    app_model_catalog_router(catalog).merge(app_model_rankings_router())
}

pub fn catalog_app_router_with_sqlite_pool<C>(pool: SqlitePool, catalog: Arc<C>) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
        Arc::new(SqliteModelRankingsReadStore::new(pool));
    catalog_app_router_with_catalog_and_read_store(catalog, read_store)
}

pub fn catalog_app_router_with_postgres_pool<C>(pool: PgPool, catalog: Arc<C>) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
        Arc::new(PostgresModelRankingsReadStore::new(pool));
    catalog_app_router_with_catalog_and_read_store(catalog, read_store)
}

pub fn catalog_app_router_with_catalog_and_read_store<C>(
    catalog: Arc<C>,
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    app_model_catalog_router(catalog).merge(app_model_rankings_router_with_read_store(read_store))
}
