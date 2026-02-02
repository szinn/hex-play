//! gRPC error mapping from core errors to tonic Status codes.

use hex_play_core::{Error as CoreError, ErrorKind};
use tonic::Status;

/// Maps a core error to the appropriate tonic Status code.
pub fn map_core_error(error: CoreError) -> Status {
    let message = error.to_string();
    match error.kind() {
        ErrorKind::NotFound => Status::not_found(message),
        ErrorKind::Conflict => Status::already_exists(message),
        ErrorKind::ValidationError => Status::invalid_argument(message),
        ErrorKind::BadRequest => Status::invalid_argument(message),
        ErrorKind::InternalError => Status::internal(message),
    }
}
