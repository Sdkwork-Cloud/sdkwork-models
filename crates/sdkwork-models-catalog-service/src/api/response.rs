use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlusApiResult<T: Serialize> {
    pub code: String,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> PlusApiResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: "2000".to_owned(),
            msg: "SUCCESS".to_owned(),
            trace_id: None,
            data: Some(data),
        }
    }

    pub fn success_with_trace_id(data: T, trace_id: impl Into<String>) -> Self {
        Self {
            code: "2000".to_owned(),
            msg: "SUCCESS".to_owned(),
            trace_id: Some(trace_id.into()),
            data: Some(data),
        }
    }
}

impl PlusApiResult<()> {
    pub fn error(code: impl Into<String>, msg: impl Into<String>) -> Self {
        let msg = msg.into();
        Self {
            code: code.into(),
            msg,
            trace_id: None,
            data: None,
        }
    }

    pub fn error_with_trace_id(
        code: impl Into<String>,
        msg: impl Into<String>,
        trace_id: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            trace_id: Some(trace_id.into()),
            data: None,
        }
    }
}
