use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use tracing::warn;

#[derive(Clone, Debug)]
pub enum ModelsReadinessProbe {
    Static,
    Sqlite(sqlx::SqlitePool),
    Postgres(sqlx::PgPool),
}

impl Default for ModelsReadinessProbe {
    fn default() -> Self {
        Self::Static
    }
}

pub fn models_health_router() -> Router {
    models_health_router_with_readiness(ModelsReadinessProbe::Static)
}

pub fn models_health_router_with_readiness(probe: ModelsReadinessProbe) -> Router {
    Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(ready_check))
        .with_state(probe)
}

async fn health_check() -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "sdkwork-models"
        })),
    )
        .into_response()
}

async fn ready_check(State(probe): State<ModelsReadinessProbe>) -> Response {
    match probe {
        ModelsReadinessProbe::Static => (
            StatusCode::OK,
            Json(json!({
                "status": "ready",
                "service": "sdkwork-models",
                "database": "not_configured"
            })),
        )
            .into_response(),
        ModelsReadinessProbe::Sqlite(pool) => match sqlx::query("SELECT 1").execute(&pool).await {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({
                    "status": "ready",
                    "service": "sdkwork-models",
                    "database": "sqlite"
                })),
            )
                .into_response(),
            Err(error) => {
                warn!(service = "sdkwork-models", database = "sqlite", %error, "readiness probe failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({
                        "status": "not_ready",
                        "service": "sdkwork-models",
                        "database": "sqlite"
                    })),
                )
                    .into_response()
            }
        },
        ModelsReadinessProbe::Postgres(pool) => {
            match sqlx::query("SELECT 1").execute(&pool).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({
                        "status": "ready",
                        "service": "sdkwork-models",
                        "database": "postgres"
                    })),
                )
                    .into_response(),
                Err(error) => {
                    warn!(service = "sdkwork-models", database = "postgres", %error, "readiness probe failed");
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "status": "not_ready",
                            "service": "sdkwork-models",
                            "database": "postgres"
                        })),
                    )
                        .into_response()
                }
            }
        }
    }
}
