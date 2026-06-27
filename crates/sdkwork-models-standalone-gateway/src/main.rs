use std::sync::Arc;

use axum::Router;
use sdkwork_models_standalone_gateway::ModelsServiceHost;
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if iam_enabled() {
        sdkwork_iam_web_adapter::prime_signing_master_secret();
        info!("IAM session resolution enabled");
    }

    let host = Arc::new(
        ModelsServiceHost::new()
            .await
            .expect("models service host bootstrap failed"),
    );

    let app = Router::new()
        .merge(host.health_router())
        .merge(host.clone().app_router_with_framework().await)
        .merge(host.backend_router_with_framework().await)
        .layer(CorsLayer::permissive());

    let addr = std::env::var("SDKWORK_MODELS_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    info!(%addr, "starting sdkwork-models-standalone-gateway");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind sdkwork-models-standalone-gateway listener");
    axum::serve(listener, app)
        .await
        .expect("serve sdkwork-models-standalone-gateway");
}

fn iam_enabled() -> bool {
    matches!(
        std::env::var("SDKWORK_MODELS_IAM_ENABLED").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}
