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

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("{0}")]
    Message(String),

    #[error("Constraint Error - {0}")]
    Constraint(String),

    #[error("Confict Error")]
    Conflict,

    #[error("Not found")]
    NotFound,

    #[error("Read-only Transaction")]
    ReadOnly,

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}
