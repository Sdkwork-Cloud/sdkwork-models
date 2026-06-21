#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestIdError {
    #[allow(dead_code)]
    Invalid(String),
    System(String),
}

pub fn generate_server_request_id() -> Result<String, RequestIdError> {
    generate_uuid_v4()
}

fn generate_uuid_v4() -> Result<String, RequestIdError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| {
        RequestIdError::System(format!("failed to generate request id: {error}"))
    })?;
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(RequestIdError::System(
            "failed to generate request id: random source returned all zero bytes".to_owned(),
        ));
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(format_uuid(&bytes))
}

#[cfg(test)]
fn is_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    bytes.iter().enumerate().all(|(index, byte)| {
        matches!(index, 8 | 13 | 18 | 23) && *byte == b'-'
            || !matches!(index, 8 | 13 | 18 | 23) && byte.is_ascii_hexdigit()
    })
}

fn format_uuid(bytes: &[u8; 16]) -> String {
    let hex = b"0123456789abcdef";
    let mut output = String::with_capacity(36);
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            output.push('-');
        }
        output.push(hex[(byte >> 4) as usize] as char);
        output.push(hex[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_server_request_id_generates_uuid() {
        let generated = generate_server_request_id().unwrap();

        assert!(is_uuid(&generated));
        assert_eq!(Some(b'4'), generated.as_bytes().get(14).copied());
        assert!(matches!(
            generated.as_bytes()[19],
            b'8' | b'9' | b'a' | b'b'
        ));
    }
}
