//! Gateway bootstrap for sdkwork-models.

use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_models_service_host::ModelsServiceHost;
use std::sync::Arc;

use crate::catalog_app_router::catalog_app_router_with_catalog_voice_and_read_store;
use crate::catalog_backend_router::{
    catalog_backend_router_with_postgres_pool_and_catalog,
    catalog_backend_router_with_sqlite_pool_and_catalog,
};

pub struct ApplicationAssembly {
    pub router: Router,
    pub database_pool: DatabasePool,
}

pub async fn assemble_application_business_router() -> Result<ApplicationAssembly, String> {
    let host = Arc::new(ModelsServiceHost::new().await?);
    let app_business = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            use sdkwork_models_catalog_repository_sqlx::PostgresModelRankingsReadStore;
            use sdkwork_models_contract_service::ModelRankingsReadModelStore;
            let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
                Arc::new(PostgresModelRankingsReadStore::new(pool.clone()));
            catalog_app_router_with_catalog_voice_and_read_store(
                host.pricing_catalog(),
                host.voice_catalog(),
                read_store,
            )
        }
        DatabasePool::Sqlite(pool, _) => {
            use sdkwork_models_catalog_repository_sqlx::SqliteModelRankingsReadStore;
            use sdkwork_models_contract_service::ModelRankingsReadModelStore;
            let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
                Arc::new(SqliteModelRankingsReadStore::new(pool.clone()));
            catalog_app_router_with_catalog_voice_and_read_store(
                host.pricing_catalog(),
                host.voice_catalog(),
                read_store,
            )
        }
    };
    let backend_business = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => catalog_backend_router_with_postgres_pool_and_catalog(
            pool.clone(),
            Some(host.voice_catalog()),
        ),
        DatabasePool::Sqlite(pool, _) => catalog_backend_router_with_sqlite_pool_and_catalog(
            pool.clone(),
            Some(host.voice_catalog()),
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
    Ok(ApplicationAssembly {
        router: Router::new().merge(app).merge(backend),
        database_pool: host.database_pool().clone(),
    })
}

pub async fn assemble_application_router() -> Result<ApplicationAssembly, String> {
    assemble_application_business_router().await
}
