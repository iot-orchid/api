use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::DecodeError;
use bcrypt::BcryptError;
use uuid::Error as UuidError;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    InvalidUuid(UuidError),
    #[allow(dead_code)]
    DecodeError(DecodeError),
    UnauthorizedClusterAccess,
    #[allow(dead_code)]
    DatabaseError(sea_orm::error::DbErr),
    IncorrectPassword,
    UsernameNotFound,
    #[allow(dead_code)]
    BcryptError(BcryptError),
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

impl From<sea_orm::error::DbErr> for Error {
    fn from(e: sea_orm::error::DbErr) -> Self {
        Error::DatabaseError(e)
    }
}

impl From<BcryptError> for Error {
    fn from(e: BcryptError) -> Self {
        Error::BcryptError(e)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidUuid(_) => StatusCode::BAD_REQUEST.into_response(),
            Error::DecodeError(_) => StatusCode::BAD_REQUEST.into_response(),
            Error::UnauthorizedClusterAccess => StatusCode::UNAUTHORIZED.into_response(),
            Error::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::IncorrectPassword => StatusCode::UNAUTHORIZED.into_response(),
            Error::UsernameNotFound => StatusCode::UNAUTHORIZED.into_response(),
            Error::BcryptError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
