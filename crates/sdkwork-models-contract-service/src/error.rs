use std::fmt::{Display, Formatter};

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    message: String,
    kind: DomainErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainErrorKind {
    System,
    Conflict,
    NotFound,
    BadRequest,
}

impl DomainError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: DomainErrorKind::System,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: DomainErrorKind::Conflict,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: DomainErrorKind::NotFound,
        }
    }

    /// A client-side validation failure: the command is well-formed but
    /// violates a business rule (e.g. references a resource that does not
    /// exist in the expected state). Surfaced as HTTP 400, not 500.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: DomainErrorKind::BadRequest,
        }
    }

    pub fn is_conflict(&self) -> bool {
        self.kind == DomainErrorKind::Conflict
    }

    pub fn is_not_found(&self) -> bool {
        self.kind == DomainErrorKind::NotFound
    }

    pub fn is_bad_request(&self) -> bool {
        self.kind == DomainErrorKind::BadRequest
    }
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DomainError {}
