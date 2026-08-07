//! Application API assembly bootstrap for sdkwork-models.

use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_models_catalog_repository_sqlx::{
    PostgresAdminAiResourceStore, PostgresModelCatalogAdminStore, PostgresModelRankingRefreshStore,
    PostgresModelRankingsReadStore,
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
        DatabasePool::Sqlite(_, _) => unreachable!(
            "models server assembly requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
        ),
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
        DatabasePool::Sqlite(_, _) => unreachable!(
            "models server assembly requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
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
