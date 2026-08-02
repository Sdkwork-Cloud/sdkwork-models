use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};

pub struct ModelsDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl ModelsDatabaseHost {
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }

    fn orchestrator(&self, applied_by: &str) -> LifecycleOrchestrator {
        LifecycleOrchestrator::new(self.pool.clone(), self.module.clone())
            .with_applied_by(applied_by)
    }

    pub async fn migrate(&self, applied_by: &str) -> Result<usize, String> {
        let orchestrator = self.orchestrator(applied_by);
        orchestrator
            .init()
            .await
            .map_err(|error| format!("models database init failed: {error}"))?;
        orchestrator
            .migrate()
            .await
            .map_err(|error| format!("models database migrate failed: {error}"))
    }

    pub async fn plan_migrations(&self) -> Result<usize, String> {
        self.orchestrator("sdkwork-models-plan")
            .plan_migrations()
            .await
            .map(|migrations| migrations.len())
            .map_err(|error| format!("models database migration plan failed: {error}"))
    }
}

/// Loads the canonical models database module around an existing pool without
/// creating framework tables, applying schema, or seeding catalog data.
///
/// Both database roles are legitimate (ENVIRONMENT_SPEC §7.2): the server role
/// (workspace PostgreSQL profile) is resolved by `DatabaseConfig::from_env`,
/// while desktop hosts connect the client-local SQLite pool through
/// `DatabaseConfig::load_client_local_from_env`. The role-aware config loader
/// guarantees server modules never receive SQLite.
pub fn connect_models_database(pool: DatabasePool) -> Result<ModelsDatabaseHost, String> {
    let app_root = resolve_app_root();
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(&app_root)
            .map_err(|error| format!("load models database module failed: {error}"))?,
    );
    DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read models database manifest failed: {error}"))?;
    Ok(ModelsDatabaseHost { pool, module })
}

/// Runtime-safe models bootstrap. The manifest default is connection-only;
/// non-production auto-migration requires an explicit environment override.
pub async fn bootstrap_models_database(pool: DatabasePool) -> Result<ModelsDatabaseHost, String> {
    let host = connect_models_database(pool)?;
    let manifest = DatabaseManifest::from_file(host.module.manifest_path())
        .map_err(|error| format!("read models database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("MODELS", &manifest);
    if options.auto_migrate {
        let environment = std::env::var("SDKWORK_MODELS_ENVIRONMENT").unwrap_or_default();
        if production_like_environment(&environment) {
            return Err(
                "production/staging runtime must not auto-migrate the models database; run the explicit lifecycle migrate command before startup"
                    .to_owned(),
            );
        }
        host.migrate("sdkwork-models-runtime").await?;
    }
    Ok(host)
}

pub async fn migrate_models_database(
    pool: DatabasePool,
    applied_by: &str,
) -> Result<(ModelsDatabaseHost, usize), String> {
    let host = connect_models_database(pool)?;
    let applied = host.migrate(applied_by).await?;
    Ok((host, applied))
}

/// Explicitly migrates the client-local SQLite models database for runtimes
/// without a lifecycle operator (desktop standalone). The module manifest
/// disables auto-migrate, so the host runtime performs the documented
/// lifecycle migrate itself before serving requests.
pub async fn migrate_client_local_models_database(applied_by: &str) -> Result<usize, String> {
    let _ = dotenvy::dotenv();
    if !sdkwork_database_config::client_local_sqlite_url_configured() {
        return Err(
            "client-local models database migration requires SDKWORK_DATABASE_SQLITE_URL"
                .to_owned(),
        );
    }
    let config = DatabaseConfig::load_client_local_from_env("MODELS")
        .map_err(|error| format!("read models database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create models database pool failed: {error}"))?;
    migrate_models_database(pool, applied_by)
        .await
        .map(|(_, applied)| applied)
}

pub async fn bootstrap_models_database_from_env() -> Result<ModelsDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    // Client-local SQLite (desktop standalone) resolves exclusively through
    // SDKWORK_DATABASE_SQLITE_URL; otherwise the workspace PostgreSQL profile
    // applies (ENVIRONMENT_SPEC §7.2).
    let config = if sdkwork_database_config::client_local_sqlite_url_configured() {
        DatabaseConfig::load_client_local_from_env("MODELS")
    } else {
        DatabaseConfig::from_env("MODELS")
    }
    .map_err(|error| format!("read models database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create models database pool failed: {error}"))?;
    bootstrap_models_database(pool).await
}

fn resolve_app_root() -> PathBuf {
    std::env::var("SDKWORK_MODELS_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        })
}

fn production_like_environment(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "production" | "prod" | "staging"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_database_config::{DatabaseEngine, DeploymentMode};

    async fn memory_pool() -> DatabasePool {
        create_pool_from_config(DatabaseConfig {
            engine: DatabaseEngine::Sqlite,
            url: "sqlite::memory:".to_owned(),
            max_connections: 1,
            mode: DeploymentMode::Standalone,
            ..DatabaseConfig::default()
        })
        .await
        .expect("create in-memory database pool")
    }

    #[tokio::test]
    async fn client_local_sqlite_pool_connects() {
        // Desktop hosts connect the declared client-local SQLite pool; the
        // role-aware config loader keeps server modules on PostgreSQL.
        let host = connect_models_database(memory_pool().await)
            .expect("client-local SQLite pool must connect");
        assert!(host.pool().as_sqlite().is_some());
    }

    #[test]
    fn production_like_environment_rejects_production_aliases() {
        assert!(production_like_environment("production"));
        assert!(production_like_environment(" PROD "));
        assert!(production_like_environment("staging"));
        assert!(!production_like_environment("development"));
    }
}
