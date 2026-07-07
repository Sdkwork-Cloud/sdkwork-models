use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

const PRODUCTION_DEFAULT_ORIGINS: &[&str] =
    &["https://models.sdkwork.com", "https://admin.sdkwork.com"];

const DEVELOPMENT_DEFAULT_ORIGINS: &[&str] = &[
    "http://localhost:5173",
    "http://127.0.0.1:5173",
    "http://localhost:8080",
    "http://127.0.0.1:8080",
];

pub fn application_cors_layer() -> CorsLayer {
    if cors_allow_any_enabled() {
        return CorsLayer::permissive();
    }

    let origins = parse_allowed_origins();
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::HeaderName::from_static("access-token"),
            axum::http::HeaderName::from_static("auth-token"),
            axum::http::HeaderName::from_static("x-sdkwork-trace-id"),
        ])
        .allow_credentials(true)
}

fn cors_allow_any_enabled() -> bool {
    matches!(
        std::env::var("SDKWORK_MODELS_CORS_ALLOW_ANY").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

fn parse_allowed_origins() -> Vec<HeaderValue> {
    if let Ok(raw) = std::env::var("SDKWORK_MODELS_CORS_ALLOWED_ORIGINS") {
        return raw
            .split(',')
            .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
            .collect();
    }

    let defaults = match std::env::var("SDKWORK_MODELS_ENVIRONMENT").as_deref() {
        Ok("production") => PRODUCTION_DEFAULT_ORIGINS,
        _ => DEVELOPMENT_DEFAULT_ORIGINS,
    };
    defaults
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_defaults_exclude_wildcard() {
        std::env::remove_var("SDKWORK_MODELS_CORS_ALLOWED_ORIGINS");
        std::env::set_var("SDKWORK_MODELS_ENVIRONMENT", "production");
        std::env::remove_var("SDKWORK_MODELS_CORS_ALLOW_ANY");

        let origins = parse_allowed_origins();
        assert_eq!(2, origins.len());
        assert!(origins
            .iter()
            .any(|value| value.to_str().ok() == Some("https://models.sdkwork.com")));
    }
}
