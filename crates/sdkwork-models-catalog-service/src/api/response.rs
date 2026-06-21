use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlusApiResult<T: Serialize> {
    pub code: String,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> PlusApiResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: "2000".to_owned(),
            msg: "SUCCESS".to_owned(),
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
            data: None,
        }
    }
}
