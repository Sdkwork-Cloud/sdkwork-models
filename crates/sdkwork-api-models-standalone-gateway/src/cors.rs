use sdkwork_web_axum::CanonicalCorsLayer;

const PRODUCTION_DEFAULT_ORIGINS: &[&str] =
    &["https://models.sdkwork.com", "https://admin.sdkwork.com"];

const DEVELOPMENT_DEFAULT_ORIGINS: &[&str] = &[
    "http://localhost:5173",
    "http://127.0.0.1:5173",
    "http://localhost:8080",
    "http://127.0.0.1:8080",
];

pub fn application_cors_layer() -> CanonicalCorsLayer {
    let development = !matches!(
        std::env::var("SDKWORK_MODELS_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_owned())
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "prod" | "production" | "staging"
    );
    let mut policy = if development {
        sdkwork_web_core::CorsPolicy::development_private_network()
    } else {
        sdkwork_web_core::CorsPolicy::default()
    };
    for origin in parse_allowed_origins() {
        if let Ok(origin) = origin.to_str() {
            if !policy
                .allowed_origins
                .iter()
                .any(|allowed| allowed == origin)
            {
                policy.allowed_origins.push(origin.to_owned());
            }
        }
    }
    for header in ["accept", "auth-token", "x-sdkwork-trace-id"] {
        if !policy
            .allowed_headers
            .iter()
            .any(|allowed| allowed == header)
        {
            policy.allowed_headers.push(header.to_owned());
        }
    }
    sdkwork_web_axum::cors_layer_from_policy(policy)
}

fn parse_allowed_origins() -> Vec<axum::http::HeaderValue> {
    if let Ok(raw) = std::env::var("SDKWORK_MODELS_CORS_ALLOWED_ORIGINS") {
        return raw
            .split(',')
            .filter_map(|origin| axum::http::HeaderValue::from_str(origin.trim()).ok())
            .collect();
    }

    let defaults = match std::env::var("SDKWORK_MODELS_ENVIRONMENT").as_deref() {
        Ok("production") => PRODUCTION_DEFAULT_ORIGINS,
        _ => DEVELOPMENT_DEFAULT_ORIGINS,
    };
    defaults
        .iter()
        .filter_map(|origin| axum::http::HeaderValue::from_str(origin).ok())
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
