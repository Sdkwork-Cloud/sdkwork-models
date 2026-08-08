use sdkwork_web_axum::CanonicalCorsLayer;

pub fn application_cors_layer() -> CanonicalCorsLayer {
    sdkwork_web_axum::cors_layer_from_policy(application_cors_policy())
}

/// Official policy construction: dev/test environments use the framework's
/// `development_private_network` semantics (loopback/private-network dev-server
/// origins); production uses the exact allowlist from the canonical
/// `SDKWORK_CORS_ALLOWED_ORIGINS` key (legacy `SDKWORK_MODELS_CORS_ALLOWED_ORIGINS`
/// still resolves with a deprecation warning). Local-only header extensions
/// required by the models frontends are kept on top of the official defaults.
fn application_cors_policy() -> sdkwork_web_core::CorsPolicy {
    let environment =
        sdkwork_web_bootstrap::web_environment_from_env(&["SDKWORK_MODELS_ENVIRONMENT"]);
    let origins = sdkwork_web_bootstrap::cors_allowed_origins_from_env(&[
        "SDKWORK_MODELS_CORS_ALLOWED_ORIGINS",
    ]);
    let mut policy = sdkwork_web_bootstrap::security_policy_for_environment(&environment, origins);
    for header in ["accept", "auth-token", "x-sdkwork-trace-id"] {
        if !policy
            .cors
            .allowed_headers
            .iter()
            .any(|allowed| allowed == header)
        {
            policy.cors.allowed_headers.push(header.to_owned());
        }
    }
    policy.cors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_sensitive_policy_building() {
        // Production: exact allowlist only, never wildcard.
        std::env::remove_var("SDKWORK_MODELS_CORS_ALLOWED_ORIGINS");
        std::env::remove_var("SDKWORK_CORS_ALLOWED_ORIGINS");
        std::env::set_var("SDKWORK_MODELS_ENVIRONMENT", "production");

        let policy = application_cors_policy();
        assert!(!policy.allow_all_origins);
        assert!(!policy
            .allowed_origins
            .iter()
            .any(|origin| origin == "*"));

        // Development (default): loopback/private-network dev-server origins.
        std::env::remove_var("SDKWORK_MODELS_ENVIRONMENT");
        let policy = application_cors_policy();
        policy
            .validate_origin_value("http://192.168.50.12:5173")
            .expect("private-network development origin");
        policy
            .validate_origin_value("https://evil.example.com")
            .expect_err("public hostname must remain rejected");
    }
}
