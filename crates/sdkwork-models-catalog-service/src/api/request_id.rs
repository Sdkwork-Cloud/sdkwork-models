use sdkwork_web_core::new_request_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestIdError {
    #[allow(dead_code)]
    Invalid(String),
    System(String),
}

/// Generates a server-side correlation id for command audit metadata.
pub fn generate_server_request_id() -> Result<String, RequestIdError> {
    Ok(new_request_id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_web_core::is_canonical_uuid;

    #[test]
    fn generate_server_request_id_generates_uuid() {
        let generated = generate_server_request_id().unwrap();
        assert!(is_canonical_uuid(&generated));
        assert_eq!(Some(b'4'), generated.as_bytes().get(14).copied());
        assert!(matches!(
            generated.as_bytes()[19],
            b'8' | b'9' | b'a' | b'b'
        ));
    }
}
