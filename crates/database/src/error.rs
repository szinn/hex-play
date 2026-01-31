use hex_play_core::RepositoryError;
use sea_orm::{DbErr, RuntimeErr};

/// PostgreSQL error codes.
/// See: https://www.postgresql.org/docs/current/errcodes-appendix.html
mod pg_error_codes {
    /// 25006: read_only_sql_transaction
    /// Raised when a write operation is attempted on a read-only transaction.
    pub const READ_ONLY_SQL_TRANSACTION: &str = "25006";
}

pub fn handle_dberr(error: DbErr) -> RepositoryError {
    if let DbErr::Query(RuntimeErr::SqlxError(sqlx_err)) | DbErr::Exec(RuntimeErr::SqlxError(sqlx_err)) = &error {
        if let Some(db_err) = sqlx_err.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == pg_error_codes::READ_ONLY_SQL_TRANSACTION {
                    return RepositoryError::ReadOnly;
                } else {
                    tracing::error!(error_code = %code, error = %error, "Database error code");
                    return RepositoryError::Message(error.to_string());
                }
            }
        }
    }

    match error.sql_err() {
        Some(error) => match error {
            sea_orm::SqlErr::UniqueConstraintViolation(msg) => RepositoryError::Constraint(msg),
            _ => {
                tracing::error!("Got sql_err {:?}", error);
                RepositoryError::Message(error.to_string())
            }
        },
        _ => {
            tracing::error!("Got DbErr {:?}", error);
            RepositoryError::Message(error.to_string())
        }
    }
}
