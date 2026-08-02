mod json_pricing_catalog;

use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_sqlx::DatabasePool;
use sdkwork_models::{load_catalog, ModelCatalog};
use sdkwork_models_catalog_repository_sqlx::ENV_MODELS_CATALOG_ROOT;
use sdkwork_models_database_host::ModelsDatabaseHost;

pub use json_pricing_catalog::JsonPricingCatalog;

pub struct ModelsServiceHost {
    database: ModelsDatabaseHost,
    pricing_catalog: Arc<JsonPricingCatalog>,
    voice_catalog: Arc<ModelCatalog>,
    models_catalog_root: String,
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
            models_catalog_root: models_catalog_root.to_string_lossy().into_owned(),
        })
    }

    pub fn database_pool(&self) -> &DatabasePool {
        self.database.pool()
    }

    pub fn models_catalog_root(&self) -> &str {
        &self.models_catalog_root
    }

    pub fn pricing_catalog(&self) -> Arc<JsonPricingCatalog> {
        Arc::clone(&self.pricing_catalog)
    }

    pub fn voice_catalog(&self) -> Arc<ModelCatalog> {
        Arc::clone(&self.voice_catalog)
    }
}

fn resolve_models_catalog_root() -> PathBuf {
    if let Ok(root) = std::env::var(ENV_MODELS_CATALOG_ROOT) {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    // The catalog root is the repository root that owns the
    // `sdkwork-models.json` manifest (its `modelsRoot` points at `models/`).
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
