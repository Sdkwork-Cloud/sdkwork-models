//! Gateway bootstrap for sdkwork-models.

use axum::Router;
use sdkwork_models_standalone_gateway::ModelsServiceHost;
use std::sync::Arc;

pub struct ApplicationAssembly {
    pub router: Router,
}

pub async fn assemble_application_business_router() -> Result<ApplicationAssembly, String> {
    let host = Arc::new(ModelsServiceHost::new().await?);
    let app = sdkwork_routes_models_catalog_app_api::wrap_router_with_web_framework_from_env(
        host.app_router(),
    )
    .await;
    let backend = sdkwork_routes_models_catalog_backend_api::wrap_router_with_web_framework_from_env(
        host.backend_router(),
    )
    .await;
    Ok(ApplicationAssembly {
        router: Router::new().merge(app).merge(backend),
    })
}

pub async fn assemble_application_router() -> Result<ApplicationAssembly, String> {
    assemble_application_business_router().await
}
