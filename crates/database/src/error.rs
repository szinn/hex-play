use sea_orm::DbErr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Can't read database configuration file")]
    CantReadConfiguration,

    #[error(transparent)]
    SeaOrm(#[from] DbErr),
}

pub fn handle_dberr(error: DbErr) -> Error {
    tracing::error!("Got DbErr {:?}", error);

    Error::SeaOrm(error)
}
