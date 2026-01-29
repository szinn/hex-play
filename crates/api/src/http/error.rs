use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hex_play_core::{Error as CoreError, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("Not found")]
    NotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Error::NotFound => (StatusCode::NOT_FOUND, "Not found".into()),
            Error::Core(core_error) => match core_error {
                CoreError::RepositoryError(repo_error) => match repo_error {
                    RepositoryError::NotFound => (StatusCode::NOT_FOUND, "Not found".into()),
                    RepositoryError::Conflict => (StatusCode::CONFLICT, "Resource conflict".into()),
                    RepositoryError::Constraint(msg) => (StatusCode::UNPROCESSABLE_ENTITY, format!("Validation error - {msg}")),
                    RepositoryError::ReadOnly => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
                    RepositoryError::Message(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error - {msg}")),
                    RepositoryError::Any(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error - {msg}")),
                },
                CoreError::Message(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error - {msg}")),
                CoreError::Any(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error - {msg}")),
            },
        };

        tracing::error!(%status, error = %self, "Request failed");

        (status, message).into_response()
    }
}
