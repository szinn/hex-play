use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hex_play_core::{Error as CoreError, ErrorKind};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("Not found")]
    NotFound,
}

fn status_code_from_error_kind(kind: ErrorKind) -> StatusCode {
    match kind {
        ErrorKind::NotFound => StatusCode::NOT_FOUND,
        ErrorKind::Conflict => StatusCode::CONFLICT,
        ErrorKind::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
        ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Error::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Error::Core(core_error) => (status_code_from_error_kind(core_error.kind()), core_error.to_string()),
        };

        tracing::error!(%status, error = %self, "Request failed");

        (status, message).into_response()
    }
}
