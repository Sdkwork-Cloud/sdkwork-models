use std::sync::Arc;

use axum::Router;
use sdkwork_models_catalog_repository_sqlx::{
    PostgresAdminAiResourceStore, PostgresModelCatalogAdminStore, PostgresModelRankingRefreshStore,
    PostgresModelRankingsReadStore, SqliteAdminAiResourceStore, SqliteModelCatalogAdminStore,
    SqliteModelRankingRefreshStore, SqliteModelRankingsReadStore,
};
use sdkwork_models_catalog_service::{
    admin_model_management_router_with_store, admin_model_rankings_router,
    admin_model_rankings_router_with_read_store_and_refresh_store,
};
use sdkwork_models_contract_service::{
    AdminAiResourceStore, ModelCatalogAdminStore, ModelRankingRefreshStore, ModelRankingsReadModelStore,
};
use sdkwork_router_models_catalog_backend_api::admin_ai_resource_router_with_store;
use sqlx::{PgPool, SqlitePool};

use crate::entity_uuid_generator::CatalogEntityUuidGenerator;

pub fn catalog_backend_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    let admin_store: Arc<dyn ModelCatalogAdminStore + Send + Sync> =
        Arc::new(SqliteModelCatalogAdminStore::new(pool.clone()));
    let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
        Arc::new(SqliteModelRankingsReadStore::new(pool.clone()));
    let refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync> =
        Arc::new(SqliteModelRankingRefreshStore::new(pool.clone()));
    let ai_resource_store: Arc<dyn AdminAiResourceStore + Send + Sync> =
        Arc::new(SqliteAdminAiResourceStore::new(pool));

    catalog_backend_router_with_stores(admin_store, read_store, refresh_store, ai_resource_store)
}

pub fn catalog_backend_router_with_postgres_pool(pool: PgPool) -> Router {
    let admin_store: Arc<dyn ModelCatalogAdminStore + Send + Sync> =
        Arc::new(PostgresModelCatalogAdminStore::new(pool.clone()));
    let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
        Arc::new(PostgresModelRankingsReadStore::new(pool.clone()));
    let refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync> =
        Arc::new(PostgresModelRankingRefreshStore::new(pool.clone()));
    let ai_resource_store: Arc<dyn AdminAiResourceStore + Send + Sync> =
        Arc::new(PostgresAdminAiResourceStore::new(pool));

    catalog_backend_router_with_stores(admin_store, read_store, refresh_store, ai_resource_store)
}

pub fn catalog_backend_router_with_stores(
    admin_store: Arc<dyn ModelCatalogAdminStore + Send + Sync>,
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
    refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync>,
    ai_resource_store: Arc<dyn AdminAiResourceStore + Send + Sync>,
) -> Router {
    admin_model_management_router_with_store(admin_store, CatalogEntityUuidGenerator::arc())
        .merge(admin_model_rankings_router_with_read_store_and_refresh_store(
            read_store,
            refresh_store,
        ))
        .merge(admin_ai_resource_router_with_store(
            ai_resource_store,
            CatalogEntityUuidGenerator::arc(),
        ))
}

pub fn catalog_backend_router_without_stores() -> Router {
    admin_model_rankings_router()
}
