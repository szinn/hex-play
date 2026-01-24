#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

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

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}
