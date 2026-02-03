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

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Invalid ID: {0}")]
    InvalidId(i64),

    #[error("Invalid page size: {0}")]
    InvalidPageSize(u64),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Failed to parse address: {0}")]
    AddressParse(String),

    #[error("Invalid transaction type")]
    InvalidTransactionType,

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("gRPC client error: {0}")]
    GrpcClientError(String),

    #[error(transparent)]
    RepositoryError(#[from] RepositoryError),

    #[cfg(any(test, feature = "test-support"))]
    #[error("Mock not configured: {0}")]
    MockNotConfigured(&'static str),
}

impl Error {
    /// Returns the error kind for HTTP response mapping.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::InvalidId(_) | Error::InvalidPageSize(_) | Error::InvalidUuid(_) => ErrorKind::BadRequest,
            Error::Validation(_) => ErrorKind::ValidationError,
            Error::AddressParse(_) | Error::InvalidTransactionType | Error::NetworkError(_) | Error::GrpcClientError(_) => ErrorKind::InternalError,
            Error::RepositoryError(e) => e.kind(),
            #[cfg(any(test, feature = "test-support"))]
            Error::MockNotConfigured(_) => ErrorKind::InternalError,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum RepositoryError {
    #[error("Constraint Error - {0}")]
    Constraint(String),

    #[error("Conflict Error")]
    Conflict,

    #[error("Not found")]
    NotFound,

    #[error("Read-only Transaction")]
    ReadOnly,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Query canceled")]
    QueryCanceled,
}

impl RepositoryError {
    /// Returns the error kind for HTTP response mapping.
    pub fn kind(&self) -> ErrorKind {
        match self {
            RepositoryError::NotFound => ErrorKind::NotFound,
            RepositoryError::Conflict => ErrorKind::Conflict,
            RepositoryError::Constraint(_) => ErrorKind::ValidationError,
            RepositoryError::ReadOnly | RepositoryError::Database(_) | RepositoryError::QueryCanceled => ErrorKind::InternalError,
        }
    }
}
