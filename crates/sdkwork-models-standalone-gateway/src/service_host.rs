use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_models_catalog_repository_sqlx::ENV_MODELS_CATALOG_ROOT;
use sdkwork_models_database_host::ModelsDatabaseHost;
use sqlx::{PgPool, SqlitePool};

use crate::{
    catalog_app_router_with_catalog_voice_and_read_store,
    catalog_backend_router_with_postgres_pool_and_catalog,
    catalog_backend_router_with_sqlite_pool_and_catalog, json_pricing_catalog::JsonPricingCatalog,
    models_health_router_with_readiness, ModelsReadinessProbe,
};
use sdkwork_models::{load_catalog, ModelCatalog};

pub struct ModelsServiceHost {
    database: ModelsDatabaseHost,
    pricing_catalog: Arc<JsonPricingCatalog>,
    voice_catalog: Arc<ModelCatalog>,
    models_catalog_root: Option<String>,
}

impl ModelsServiceHost {
    pub async fn new() -> Result<Self, String> {
        let _ = dotenvy::dotenv();
        let database = sdkwork_models_database_host::bootstrap_models_database_from_env().await?;
        let models_catalog_root = resolve_models_catalog_root();
        let voice_catalog = Arc::new(
            load_catalog(&models_catalog_root)
                .map_err(|error| format!("load catalog JSON failed: {error}"))?,
        );
        let pricing_catalog = Arc::new(JsonPricingCatalog::from_catalog(voice_catalog.as_ref()));
        Ok(Self {
            database,
            pricing_catalog,
            voice_catalog,
            models_catalog_root: Some(
                models_catalog_root
                    .to_string_lossy()
                    .into_owned(),
            ),
        })
    }

    pub fn models_catalog_root(&self) -> Option<&str> {
        self.models_catalog_root.as_deref()
    }

    pub fn pricing_catalog(&self) -> Arc<JsonPricingCatalog> {
        Arc::clone(&self.pricing_catalog)
    }

    pub fn health_router(&self) -> Router {
        match self.database.pool() {
            DatabasePool::Postgres(pool, _) => models_health_router_with_readiness(
                ModelsReadinessProbe::Postgres(pool.clone()),
            ),
            DatabasePool::Sqlite(pool, _) => models_health_router_with_readiness(
                ModelsReadinessProbe::Sqlite(pool.clone()),
            ),
        }
    }

    pub fn backend_router(&self) -> Router {
        let voice_catalog = Some(Arc::clone(&self.voice_catalog));
        match self.database.pool() {
            DatabasePool::Postgres(pool, _) => {
                catalog_backend_router_with_postgres_pool_and_catalog(pool.clone(), voice_catalog)
            }
            DatabasePool::Sqlite(pool, _) => {
                catalog_backend_router_with_sqlite_pool_and_catalog(pool.clone(), voice_catalog)
            }
        }
    }

    pub fn app_router(&self) -> Router {
        match self.database.pool() {
            DatabasePool::Postgres(pool, _) => {
                self.app_router_with_postgres_pool(pool.clone())
            }
            DatabasePool::Sqlite(pool, _) => self.app_router_with_sqlite_pool(pool.clone()),
        }
    }

    pub async fn backend_router_with_framework(self: Arc<Self>) -> Router {
        sdkwork_routes_models_catalog_backend_api::wrap_router_with_web_framework_from_env(
            self.backend_router(),
        )
        .await
    }

    pub async fn app_router_with_framework(self: Arc<Self>) -> Router {
        sdkwork_routes_models_catalog_app_api::wrap_router_with_web_framework_from_env(self.app_router())
            .await
    }

    fn app_router_with_sqlite_pool(&self, pool: SqlitePool) -> Router {
        use sdkwork_models_catalog_repository_sqlx::SqliteModelRankingsReadStore;
        use sdkwork_models_contract_service::ModelRankingsReadModelStore;

        let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
            Arc::new(SqliteModelRankingsReadStore::new(pool));
        catalog_app_router_with_catalog_voice_and_read_store(
            self.pricing_catalog(),
            Arc::clone(&self.voice_catalog),
            read_store,
        )
    }

    fn app_router_with_postgres_pool(&self, pool: PgPool) -> Router {
        use sdkwork_models_catalog_repository_sqlx::PostgresModelRankingsReadStore;
        use sdkwork_models_contract_service::ModelRankingsReadModelStore;

        let read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync> =
            Arc::new(PostgresModelRankingsReadStore::new(pool));
        catalog_app_router_with_catalog_voice_and_read_store(
            self.pricing_catalog(),
            Arc::clone(&self.voice_catalog),
            read_store,
        )
    }
}

fn resolve_models_catalog_root() -> PathBuf {
    if let Ok(root) = std::env::var(ENV_MODELS_CATALOG_ROOT) {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../models")
}
