use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkProblemDetail, SdkWorkResultCode};
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

    pub fn error(code: impl AsRef<str>, message: impl Into<String>) -> ProblemResponse {
        ProblemResponse::from_legacy(code.as_ref(), message.into(), new_trace_id())
    }

    pub fn error_for(
        ctx: &WebRequestContext,
        code: impl AsRef<str>,
        message: impl Into<String>,
    ) -> ProblemResponse {
        ProblemResponse::from_legacy(
            code.as_ref(),
            message.into(),
            trace_id_from_context(Some(ctx)),
        )
    }
}

/// Builds a ProblemDetail response, optionally overriding HTTP status for legacy handler paths.
pub fn legacy_problem(
    status: StatusCode,
    code: impl AsRef<str>,
    message: impl Into<String>,
) -> Response {
    let mut response = ApiResponse::error(code, message).into_response();
    *response.status_mut() = status;
    response
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
    code: impl AsRef<str>,
    message: impl Into<String>,
) -> Response {
    ApiResponse::error_for(ctx, code, message).into_response()
}

#[derive(Debug, Clone)]
pub struct ProblemResponse {
    pub problem: SdkWorkProblemDetail,
}

impl ProblemResponse {
    pub fn from_legacy(legacy_code: &str, message: String, trace_id: String) -> Self {
        let result_code = map_legacy_wire_code(legacy_code);
        Self {
            problem: SdkWorkProblemDetail::platform(result_code, message, trace_id),
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
    if let Ok(value) = HeaderValue::from_str(trace_id) {
        response.headers_mut().insert(
            HeaderName::from_static("x-sdkwork-trace-id"),
            value,
        );
    }
}

fn map_legacy_wire_code(legacy_code: &str) -> SdkWorkResultCode {
    match legacy_code.trim() {
        "4001" | "4004" => SdkWorkResultCode::ValidationError,
        "4010" => SdkWorkResultCode::AuthenticationRequired,
        "4040" => SdkWorkResultCode::NotFound,
        "4090" => SdkWorkResultCode::Conflict,
        "4220" => SdkWorkResultCode::UnprocessableEntity,
        "5000" => SdkWorkResultCode::InternalError,
        "5030" => SdkWorkResultCode::ServiceUnavailable,
        "not_found" => SdkWorkResultCode::NotFound,
        "invalid_input" | "validation_error" => SdkWorkResultCode::ValidationError,
        "forbidden" => SdkWorkResultCode::PermissionRequired,
        "conflict" => SdkWorkResultCode::Conflict,
        "rate_limited" => SdkWorkResultCode::RateLimitExceeded,
        "provider_error" => SdkWorkResultCode::BadGateway,
        _ => SdkWorkResultCode::InternalError,
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
    fn legacy_error_maps_to_problem_detail() {
        let response = ApiResponse::error("4040", "missing resource");
        assert_eq!(404, response.problem.status);
        assert_eq!(40401, response.problem.code);
    }

    #[test]
    fn finish_success_attaches_trace_header() {
        let response = finish_success(&test_context(), serde_json::json!({"item": 1}));
        assert!(response.headers().get("x-sdkwork-trace-id").is_some());
    }
}
