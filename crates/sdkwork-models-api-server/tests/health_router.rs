use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sdkwork_models_api_server::models_health_router;
use tower::ServiceExt;

#[tokio::test]
async fn healthz_returns_ok_envelope() {
    let app = models_health_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["service"], "sdkwork-models");
}

#[tokio::test]
async fn readyz_returns_ready_envelope() {
    let app = models_health_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["service"], "sdkwork-models");
}
