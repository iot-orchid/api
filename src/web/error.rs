use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::DecodeError;
use uuid::Error as UuidError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidUuid(UuidError),
    DecodeError(DecodeError),
}

impl From<UuidError> for Error {
    fn from(e: UuidError) -> Self {
        Error::InvalidUuid(e)
    }
}

impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Self {
        Error::DecodeError(e)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidUuid(_) => StatusCode::BAD_REQUEST.into_response(),
            Error::DecodeError(_) => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}
