use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_claw_http::{TrustedRequestSubject, TrustedRequestSubjectError};

use crate::api::response::PlusApiResult;

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
    subject: Option<TrustedRequestSubject>,
    require_subject: bool,
    map: impl FnOnce(TrustedRequestSubject) -> T,
) -> Result<Option<T>, Response> {
    match optional_subject_or_unauthorized(subject, require_subject)? {
        Some(trusted) => Ok(Some(map(trusted))),
        None => Ok(None),
    }
}

pub fn unauthorized_subject_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(PlusApiResult::<()>::error(
            "4010",
            TrustedRequestSubjectError::MissingExtension.to_string(),
        )),
    )
        .into_response()
}

pub fn required_subject(
    subject: Option<TrustedRequestSubject>,
) -> Result<TrustedRequestSubject, Response> {
    subject.ok_or_else(unauthorized_subject_response)
}

pub fn optional_subject_or_unauthorized(
    subject: Option<TrustedRequestSubject>,
    require_subject: bool,
) -> Result<Option<TrustedRequestSubject>, Response> {
    match subject {
        Some(subject) => Ok(Some(subject)),
        None if require_subject => Err(unauthorized_subject_response()),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use sdkwork_claw_http::TrustedRequestSubject;

    fn sample_subject() -> TrustedRequestSubject {
        TrustedRequestSubject {
            tenant_id: 100001,
            organization_id: 1,
            user_id: 42,
            operator_id: 42,
            operator_type: 1,
        }
    }

    #[test]
    fn optional_subject_requires_auth_when_flag_enabled() {
        let result = optional_subject_or_unauthorized(None, true);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn optional_subject_unauthorized_response_is_401() {
        let response = optional_subject_or_unauthorized(None, true).expect_err("401");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(payload["code"], "4010");
    }

    #[test]
    fn optional_subject_allows_anonymous_when_flag_disabled() {
        let result = optional_subject_or_unauthorized(None, false).expect("anonymous");
        assert!(result.is_none());
    }

    #[test]
    fn optional_subject_passes_through_present_subject() {
        let subject = sample_subject();
        let result = optional_subject_or_unauthorized(Some(subject.clone()), true)
            .expect("subject")
            .expect("some");
        assert_eq!(result.tenant_id, subject.tenant_id);
    }
}
