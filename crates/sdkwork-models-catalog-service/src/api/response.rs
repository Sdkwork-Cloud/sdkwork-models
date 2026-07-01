use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_utils_rust::{
    SdkWorkApiResponse, SdkWorkProblemDetail, SdkWorkResultCode, SDKWORK_TRACE_ID_HEADER,
};
use sdkwork_web_core::WebRequestContext;
use serde::Serialize;

use crate::api::request_id::generate_server_request_id;

/// Handler response helpers aligned with `SdkWorkApiResponse` / `ProblemDetail`.
pub struct ApiResponse;

pub fn new_trace_id() -> String {
    generate_server_request_id().unwrap_or_else(|_| {
        "00000000-0000-4000-8000-000000000000".to_string()
    })
}

pub fn trace_id_from_context(ctx: Option<&WebRequestContext>) -> String {
    ctx.map(WebRequestContext::resolved_trace_id)
        .unwrap_or_else(new_trace_id)
}

impl ApiResponse {
    pub fn success<T: Serialize>(data: T) -> SdkWorkApiResponse<T> {
        SdkWorkApiResponse::success(data, new_trace_id())
    }

    pub fn success_for<T: Serialize>(ctx: &WebRequestContext, data: T) -> SdkWorkApiResponse<T> {
        SdkWorkApiResponse::success(data, trace_id_from_context(Some(ctx)))
    }

    pub fn error(code: SdkWorkResultCode, message: impl Into<String>) -> ProblemResponse {
        ProblemResponse::from_code(code, message.into(), new_trace_id())
    }

    pub fn error_for(
        ctx: &WebRequestContext,
        code: SdkWorkResultCode,
        message: impl Into<String>,
    ) -> ProblemResponse {
        ProblemResponse::from_code(
            code,
            message.into(),
            trace_id_from_context(Some(ctx)),
        )
    }
}

/// Builds a request-scoped ProblemDetail response for a platform result code.
pub fn problem_for(
    ctx: &WebRequestContext,
    code: SdkWorkResultCode,
    message: impl Into<String>,
) -> Response {
    ApiResponse::error_for(ctx, code, message).into_response()
}

pub fn finish_success<T: Serialize>(ctx: &WebRequestContext, data: T) -> Response {
    let trace_id = trace_id_from_context(Some(ctx));
    let envelope = SdkWorkApiResponse::success(data, trace_id.clone());
    let mut response = Json(envelope).into_response();
    attach_trace_header(&mut response, &trace_id);
    response
}

pub fn finish_error(
    ctx: &WebRequestContext,
    code: SdkWorkResultCode,
    message: impl Into<String>,
) -> Response {
    problem_for(ctx, code, message)
}

#[derive(Debug, Clone)]
pub struct ProblemResponse {
    pub problem: SdkWorkProblemDetail,
}

impl ProblemResponse {
    pub fn from_code(code: SdkWorkResultCode, message: String, trace_id: String) -> Self {
        Self {
            problem: SdkWorkProblemDetail::platform(code, message, trace_id),
        }
    }
}

impl IntoResponse for ProblemResponse {
    fn into_response(self) -> Response {
        let trace_id = self.problem.trace_id.clone();
        let status = StatusCode::from_u16(self.problem.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, Json(self.problem)).into_response();
        attach_trace_header(&mut response, &trace_id);
        response
    }
}

pub fn attach_trace_header(response: &mut Response, trace_id: &str) {
    if let Ok(header_name) = HeaderName::try_from(SDKWORK_TRACE_ID_HEADER) {
        if let Ok(value) = HeaderValue::from_str(trace_id) {
            response.headers_mut().insert(header_name, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_web_core::{
        ServerRequestId, WebApiSurface, WebAuthMode, WebRequestContext, WebTransportFacts,
    };

    fn test_context() -> WebRequestContext {
        WebRequestContext {
            request_id: ServerRequestId("test-req".to_owned()),
            api_surface: WebApiSurface::BackendApi,
            auth_mode: WebAuthMode::DualToken,
            principal: None,
            transport: WebTransportFacts {
                path: "/backend/v3/api/ai/models".to_owned(),
                method: "GET".to_owned(),
                auth_token_present: true,
                access_token_present: true,
                api_key_present: false,
                oauth_bearer_present: false,
                agent_token_present: false,
            },
            locale: None,
            client_kind: None,
            operation: None,
            trace_id: Some("trace-from-context-abc".to_owned()),
        }
    }

    #[test]
    fn success_envelope_uses_sdkwork_v3_shape() {
        let body = ApiResponse::success(serde_json::json!({"items": []}));
        assert_eq!(0, body.code);
        assert!(!body.trace_id.is_empty());
    }

    #[test]
    fn success_for_context_uses_resolved_trace_id() {
        let body = ApiResponse::success_for(&test_context(), serde_json::json!({"item": 1}));
        assert_eq!("trace-from-context-abc", body.trace_id);
    }

    #[test]
    fn platform_error_uses_problem_detail() {
        let response = ApiResponse::error(SdkWorkResultCode::NotFound, "missing resource");
        assert_eq!(404, response.problem.status);
        assert_eq!(40401, response.problem.code);
    }

    #[test]
    fn finish_success_attaches_trace_header() {
        let response = finish_success(&test_context(), serde_json::json!({"item": 1}));
        assert!(response.headers().get("x-sdkwork-trace-id").is_some());
    }

    #[test]
    fn problem_for_uses_context_trace_id() {
        let ctx = test_context();
        let response = problem_for(&ctx, SdkWorkResultCode::ValidationError, "invalid input");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("x-sdkwork-trace-id").and_then(|v| v.to_str().ok()),
            Some("trace-from-context-abc")
        );
    }
}
