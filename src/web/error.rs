use axum::{
    http::{method, StatusCode},
    response::{IntoResponse, Response},
};
use base64::DecodeError;
use bcrypt::BcryptError;
use uuid::Error as UuidError;
pub type Result<T> = std::result::Result<T, Error>;
use crate::auth;
#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    InvalidUuid(UuidError),
    DecodeError(DecodeError),
    UnauthorizedClusterAccess,
    DatabaseError(sea_orm::error::DbErr),
    IncorrectPassword,
    UsernameNotFound,
    BcryptError(BcryptError),
    JwtError(auth::error::Error),
    InvalidHeader(axum::http::header::InvalidHeaderValue),
    AxumHttpError(axum::http::Error),
    ExpectedCookiesNotFound,
    InvalidTopicFormat,
    Unauthorized,
    SerdeJson(serde_json::Error),
    InvalidMethod(String),
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

impl From<auth::error::Error> for Error {
    fn from(e: auth::error::Error) -> Self {
        Error::JwtError(e)
    }
}

impl From<axum::http::header::InvalidHeaderValue> for Error {
    fn from(e: axum::http::header::InvalidHeaderValue) -> Self {
        Error::InvalidHeader(e)
    }
}

impl From<axum::http::Error> for Error {
    fn from(e: axum::http::Error) -> Self {
        Error::AxumHttpError(e)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Error::SerdeJson(e)
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
            Error::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::InvalidHeader(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::AxumHttpError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::ExpectedCookiesNotFound => StatusCode::BAD_REQUEST.into_response(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Error::InvalidTopicFormat => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::SerdeJson(e) => {
                // add the message to the response body
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            },
            Error::InvalidMethod(e) => {
                // add the message to the response body
                (StatusCode::BAD_REQUEST, format!("'{}' is an invalid method", e)).into_response()
            },
        }
    }
}
