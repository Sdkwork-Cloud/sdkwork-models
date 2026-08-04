use axum::response::Response;
use sdkwork_cloudrouter_http::{TrustedRequestSubject, TrustedRequestSubjectError};
use sdkwork_utils_rust::SdkWorkResultCode;
use sdkwork_web_core::WebRequestContext;

use crate::api::response::problem_for;

#[derive(Debug, Clone, Copy)]
pub struct AdminOperatorFields {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub operator_id: i64,
    pub operator_type: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct AppUserFields {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub user_id: i64,
}

pub fn admin_operator_fields(trusted: TrustedRequestSubject) -> AdminOperatorFields {
    AdminOperatorFields {
        tenant_id: trusted.tenant_id,
        organization_id: trusted.organization_id,
        operator_id: trusted.operator_id,
        operator_type: trusted.operator_type,
    }
}

pub fn app_user_fields(trusted: TrustedRequestSubject) -> AppUserFields {
    AppUserFields {
        tenant_id: trusted.tenant_id,
        organization_id: trusted.organization_id,
        user_id: trusted.user_id,
    }
}

pub fn map_optional_app_user_subject<T>(
    ctx: &WebRequestContext,
    subject: Option<TrustedRequestSubject>,
    require_subject: bool,
    map: impl FnOnce(TrustedRequestSubject) -> T,
) -> Result<Option<T>, Response> {
    match optional_subject_or_unauthorized(ctx, subject, require_subject)? {
        Some(trusted) => Ok(Some(map(trusted))),
        None => Ok(None),
    }
}

pub fn unauthorized_subject_response(ctx: &WebRequestContext) -> Response {
    problem_for(
        ctx,
        SdkWorkResultCode::AuthenticationRequired,
        TrustedRequestSubjectError::MissingExtension.to_string(),
    )
}

pub fn required_subject(
    ctx: &WebRequestContext,
    subject: Option<TrustedRequestSubject>,
) -> Result<TrustedRequestSubject, Response> {
    subject.ok_or_else(|| unauthorized_subject_response(ctx))
}

pub fn optional_subject_or_unauthorized(
    ctx: &WebRequestContext,
    subject: Option<TrustedRequestSubject>,
    require_subject: bool,
) -> Result<Option<TrustedRequestSubject>, Response> {
    match subject {
        Some(subject) => Ok(Some(subject)),
        None if require_subject => Err(unauthorized_subject_response(ctx)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use sdkwork_cloudrouter_http::TrustedRequestSubject;
    use sdkwork_web_core::{ServerRequestId, WebApiSurface, WebAuthMode, WebTransportFacts};

    fn test_context() -> WebRequestContext {
        WebRequestContext {
            request_id: ServerRequestId("test-req".to_owned()),
            api_surface: WebApiSurface::AppApi,
            auth_mode: WebAuthMode::DualToken,
            principal: None,
            transport: WebTransportFacts {
                path: "/app/v3/api/ai/model_rankings".to_owned(),
                method: "GET".to_owned(),
                auth_token_present: true,
                access_token_present: true,
                api_key_present: false,
                ingress_token_present: false,
                oauth_bearer_present: false,
                agent_token_present: false,
            },
            locale: None,
            client_kind: None,
            operation: None,
            trace_id: Some("trace-from-context-abc".to_owned()),
            idempotency_key: None,
        }
    }

    fn sample_subject() -> TrustedRequestSubject {
        TrustedRequestSubject {
            tenant_id: 100001,
            organization_id: 0,
            user_id: 42,
            operator_id: 42,
            operator_type: 1,
        }
    }

    #[test]
    fn optional_subject_requires_auth_when_flag_enabled() {
        let ctx = test_context();
        let result = optional_subject_or_unauthorized(&ctx, None, true);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn optional_subject_unauthorized_response_is_401() {
        let ctx = test_context();
        let response = optional_subject_or_unauthorized(&ctx, None, true).expect_err("401");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(payload["code"], 40101);
        assert_eq!(payload["traceId"].as_str(), Some("trace-from-context-abc"));
    }

    #[test]
    fn optional_subject_allows_anonymous_when_flag_disabled() {
        let ctx = test_context();
        let result = optional_subject_or_unauthorized(&ctx, None, false).expect("anonymous");
        assert!(result.is_none());
    }

    #[test]
    fn optional_subject_passes_through_present_subject() {
        let ctx = test_context();
        let subject = sample_subject();
        let result = optional_subject_or_unauthorized(&ctx, Some(subject.clone()), true)
            .expect("subject")
            .expect("some");
        assert_eq!(result.tenant_id, subject.tenant_id);
    }
}
