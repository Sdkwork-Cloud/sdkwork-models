use std::sync::Arc;

use axum::Router;
use sdkwork_models::ModelCatalog;
use sdkwork_models_catalog_repository_sqlx::{
    PostgresAdminAiResourceStore, PostgresModelCatalogAdminStore, PostgresModelRankingRefreshStore,
    PostgresModelRankingsReadStore, SqliteAdminAiResourceStore, SqliteModelCatalogAdminStore,
    SqliteModelRankingRefreshStore, SqliteModelRankingsReadStore,
};
use sdkwork_models_catalog_service::{
    admin_model_management_router_with_store, admin_model_rankings_router,
    admin_model_rankings_router_with_read_store_and_refresh_store,
    backend_video_profile_catalog_router, backend_voice_catalog_router,
};
use sdkwork_models_contract_service::{
    AdminAiResourceStore, ModelCatalogAdminStore, ModelRankingRefreshStore,
    ModelRankingsReadModelStore,
};
use sdkwork_routes_models_catalog_backend_api::admin_ai_resource_router_with_store;
use sqlx::{PgPool, SqlitePool};

use crate::entity_uuid_generator::CatalogEntityUuidGenerator;

pub fn catalog_backend_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    catalog_backend_router_with_sqlite_pool_and_catalog(pool, None)
}

pub fn catalog_backend_router_with_sqlite_pool_and_catalog(
    pool: SqlitePool,
    voice_catalog: Option<Arc<ModelCatalog>>,
) -> Router {
    let admin_store: Arc<dyn ModelCatalogAdminStore + Send + Sync> =
        Arc::new(SqliteModelCatalogAdminStore::new(pool.clone()));
    let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
        Arc::new(SqliteModelRankingsReadStore::new(pool.clone()));
    let refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync> =
        Arc::new(SqliteModelRankingRefreshStore::new(pool.clone()));
    let ai_resource_store: Arc<dyn AdminAiResourceStore + Send + Sync> =
        Arc::new(SqliteAdminAiResourceStore::new(pool));

    catalog_backend_router_with_stores(
        admin_store,
        read_store,
        refresh_store,
        ai_resource_store,
        voice_catalog,
    )
}

pub fn catalog_backend_router_with_postgres_pool(pool: PgPool) -> Router {
    catalog_backend_router_with_postgres_pool_and_catalog(pool, None)
}

pub fn catalog_backend_router_with_postgres_pool_and_catalog(
    pool: PgPool,
    voice_catalog: Option<Arc<ModelCatalog>>,
) -> Router {
    let admin_store: Arc<dyn ModelCatalogAdminStore + Send + Sync> =
        Arc::new(PostgresModelCatalogAdminStore::new(pool.clone()));
    let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
        Arc::new(PostgresModelRankingsReadStore::new(pool.clone()));
    let refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync> =
        Arc::new(PostgresModelRankingRefreshStore::new(pool.clone()));
    let ai_resource_store: Arc<dyn AdminAiResourceStore + Send + Sync> =
        Arc::new(PostgresAdminAiResourceStore::new(pool));

    catalog_backend_router_with_stores(
        admin_store,
        read_store,
        refresh_store,
        ai_resource_store,
        voice_catalog,
    )
}

pub fn catalog_backend_router_with_stores(
    admin_store: Arc<dyn ModelCatalogAdminStore + Send + Sync>,
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
    refresh_store: Arc<dyn ModelRankingRefreshStore + Send + Sync>,
    ai_resource_store: Arc<dyn AdminAiResourceStore + Send + Sync>,
    voice_catalog: Option<Arc<ModelCatalog>>,
) -> Router {
    let mut router =
        admin_model_management_router_with_store(admin_store, CatalogEntityUuidGenerator::arc())
            .merge(
                admin_model_rankings_router_with_read_store_and_refresh_store(
                    read_store,
                    refresh_store,
                ),
            )
            .merge(admin_ai_resource_router_with_store(
                ai_resource_store,
                CatalogEntityUuidGenerator::arc(),
            ));
    if let Some(catalog) = voice_catalog {
        router = router
            .merge(backend_voice_catalog_router(Arc::clone(&catalog)))
            .merge(backend_video_profile_catalog_router(catalog));
    }
    router
}

pub fn catalog_backend_router_without_stores() -> Router {
    admin_model_rankings_router()
}
