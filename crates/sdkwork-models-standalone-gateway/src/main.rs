use axum::Router;
use sdkwork_models_gateway_assembly::assemble_application_router;
use sdkwork_models_standalone_gateway::{
    application_cors_layer, models_health_router_with_database_pool,
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if iam_enabled() {
        info!("IAM session resolution enabled");
    }

    let assembly = assemble_application_router()
        .await
        .expect("models gateway assembly failed");

    let app = Router::new()
        .merge(models_health_router_with_database_pool(
            &assembly.database_pool,
        ))
        .merge(assembly.router)
        .layer(application_cors_layer());

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
