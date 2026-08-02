//! Application API assembly bootstrap for sdkwork-models.

use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_models_catalog_repository_sqlx::{
    PostgresAdminAiResourceStore, PostgresModelCatalogAdminStore, PostgresModelRankingRefreshStore,
    PostgresModelRankingsReadStore, SqliteAdminAiResourceStore, SqliteModelCatalogAdminStore,
    SqliteModelRankingRefreshStore, SqliteModelRankingsReadStore,
};
use sdkwork_models_contract_service::{
    AdminAiResourceStore, EntityUuidGenerator, ModelCatalogAdminStore, ModelRankingRefreshStore,
    ModelRankingsReadModelStore,
};
use sdkwork_models_service_host::ModelsServiceHost;
use std::sync::Arc;

use crate::entity_uuid_generator::CatalogEntityUuidGenerator;

pub struct ApiAssembly {
    pub router: Router,
    pub database_pool: DatabasePool,
}

pub async fn assemble_business_routes() -> Result<ApiAssembly, String> {
    let host = Arc::new(ModelsServiceHost::new().await?);
    let entity_uuid_generator: Arc<dyn EntityUuidGenerator + Send + Sync> =
        CatalogEntityUuidGenerator::arc();
    let app_business = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
                Arc::new(PostgresModelRankingsReadStore::new(pool.clone()));
            sdkwork_routes_models_catalog_app_api::gateway_mount(
                host.pricing_catalog(),
                host.voice_catalog(),
                read_store,
                Arc::new(PostgresModelCatalogAdminStore::new(pool.clone()))
                    as Arc<dyn ModelCatalogAdminStore + Send + Sync>,
                Arc::new(PostgresAdminAiResourceStore::new(pool.clone()))
                    as Arc<dyn AdminAiResourceStore + Send + Sync>,
                Arc::clone(&entity_uuid_generator),
            )
        }
        DatabasePool::Sqlite(pool, _) => {
            let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
                Arc::new(SqliteModelRankingsReadStore::new(pool.clone()));
            let resource_store = Arc::new(SqliteAdminAiResourceStore::new(pool.clone()));
            resource_store
                .initialize_schema()
                .await
                .map_err(|error| format!("initialize client-local AI resource schema failed: {error}"))?;
            sdkwork_routes_models_catalog_app_api::gateway_mount(
                host.pricing_catalog(),
                host.voice_catalog(),
                read_store,
                Arc::new(SqliteModelCatalogAdminStore::new(pool.clone()))
                    as Arc<dyn ModelCatalogAdminStore + Send + Sync>,
                resource_store as Arc<dyn AdminAiResourceStore + Send + Sync>,
                Arc::clone(&entity_uuid_generator),
            )
        }
    };
    let backend_business = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            sdkwork_routes_models_catalog_backend_api::gateway_mount(
                Arc::new(PostgresModelCatalogAdminStore::new(pool.clone()))
                    as Arc<dyn ModelCatalogAdminStore + Send + Sync>,
                Arc::new(PostgresModelRankingsReadStore::new(pool.clone()))
                    as Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
                Arc::new(PostgresModelRankingRefreshStore::new(pool.clone()))
                    as Arc<dyn ModelRankingRefreshStore + Send + Sync>,
                Arc::new(PostgresAdminAiResourceStore::new(pool.clone()))
                    as Arc<dyn AdminAiResourceStore + Send + Sync>,
                Arc::clone(&entity_uuid_generator),
                host.voice_catalog(),
            )
        }
        DatabasePool::Sqlite(pool, _) => sdkwork_routes_models_catalog_backend_api::gateway_mount(
            Arc::new(SqliteModelCatalogAdminStore::new(pool.clone()))
                as Arc<dyn ModelCatalogAdminStore + Send + Sync>,
            Arc::new(SqliteModelRankingsReadStore::new(pool.clone()))
                as Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
            Arc::new(SqliteModelRankingRefreshStore::new(pool.clone()))
                as Arc<dyn ModelRankingRefreshStore + Send + Sync>,
            Arc::new(SqliteAdminAiResourceStore::new(pool.clone()))
                as Arc<dyn AdminAiResourceStore + Send + Sync>,
            entity_uuid_generator,
            host.voice_catalog(),
        ),
    };
    let app = sdkwork_routes_models_catalog_app_api::wrap_router_with_web_framework_from_env(
        app_business,
    )
    .await;
    let backend =
        sdkwork_routes_models_catalog_backend_api::wrap_router_with_web_framework_from_env(
            backend_business,
        )
        .await;
    Ok(ApiAssembly {
        router: Router::new().merge(app).merge(backend),
        database_pool: host.database_pool().clone(),
    })
}

pub async fn assemble_api_router() -> Result<ApiAssembly, String> {
    assemble_business_routes().await
}
