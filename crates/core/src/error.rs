/// Categorizes errors for HTTP response mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Resource not found (404)
    NotFound,
    /// Resource conflict, e.g., optimistic locking failure (409)
    Conflict,
    /// Validation or constraint error (422)
    ValidationError,
    /// Bad request, e.g., invalid ID (400)
    BadRequest,
    /// Internal server error (500)
    InternalError,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("Invalid ID: {0}")]
    InvalidId(i64),

    #[error("Invalid page size: {0}")]
    InvalidPageSize(u64),

    #[error(transparent)]
    RepositoryError(#[from] RepositoryError),

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    /// Returns the error kind for HTTP response mapping.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::Message(_) | Error::Any(_) => ErrorKind::InternalError,
            Error::InvalidId(_) | Error::InvalidPageSize(_) => ErrorKind::BadRequest,
            Error::RepositoryError(e) => e.kind(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("{0}")]
    Message(String),

    #[error("Constraint Error - {0}")]
    Constraint(String),

    #[error("Conflict Error")]
    Conflict,

    #[error("Not found")]
    NotFound,

    #[error("Read-only Transaction")]
    ReadOnly,

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl RepositoryError {
    /// Returns the error kind for HTTP response mapping.
    pub fn kind(&self) -> ErrorKind {
        match self {
            RepositoryError::NotFound => ErrorKind::NotFound,
            RepositoryError::Conflict => ErrorKind::Conflict,
            RepositoryError::Constraint(_) => ErrorKind::ValidationError,
            RepositoryError::Message(_) | RepositoryError::ReadOnly | RepositoryError::Any(_) => ErrorKind::InternalError,
        }
    }
}
