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

    pub fn is_conflict(&self) -> bool {
        self.kind == DomainErrorKind::Conflict
    }

    pub fn is_not_found(&self) -> bool {
        self.kind == DomainErrorKind::NotFound
    }
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DomainError {}
