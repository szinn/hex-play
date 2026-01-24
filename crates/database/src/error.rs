use hex_play_core::RepositoryError;
use sea_orm::DbErr;

pub fn handle_dberr(error: DbErr) -> RepositoryError {
    match error.sql_err() {
        Some(error) => match error {
            sea_orm::SqlErr::UniqueConstraintViolation(msg) => RepositoryError::Constraint(msg),
            _ => {
                tracing::error!("Got sql_err {:?}", error);
                dbg!(&error);
                RepositoryError::Message(error.to_string())
            }
        },
        _ => {
            tracing::error!("Got DbErr {:?}", error);
            RepositoryError::Message(error.to_string())
        }
    }
}
