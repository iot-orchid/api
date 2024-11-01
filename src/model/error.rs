use axum::response::IntoResponse;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum ErrorKind {
    DatabaseError(sea_orm::error::DbErr),
    UuidError(uuid::Error),
    Base64DecodeError(base64::DecodeError),
    UnauthorizedClusterAccess,
    ClusterNotFound,
    MicrodeviceNotFound,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl std::error::Error for Error {}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorKind::DatabaseError(e) => write!(f, "Database error: {}", e),
            ErrorKind::UuidError(e) => write!(f, "Uuid error: {}", e),
            ErrorKind::Base64DecodeError(e) => write!(f, "Base64 decode error: {}", e),
            ErrorKind::UnauthorizedClusterAccess => write!(f, "Unauthorized cluster access"),
            ErrorKind::ClusterNotFound => write!(f, "Cluster not found"),
            ErrorKind::MicrodeviceNotFound => write!(f, "Microdevice not found"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl From<sea_orm::error::DbErr> for Error {
    fn from(e: sea_orm::error::DbErr) -> Self {
        let msg = e.to_string();
        Error {
            kind: ErrorKind::DatabaseError(e),
            message: msg,
        }
    }
}

impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Self {
        Error {
            kind: ErrorKind::UuidError(e),
            message: "Uuid error".to_string(),
        }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error {
            kind: ErrorKind::Base64DecodeError(e),
            message: "Base64 decode error".to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        axum::http::Response::builder()
            .status(match self.kind {
                ErrorKind::DatabaseError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                ErrorKind::UuidError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                ErrorKind::Base64DecodeError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                ErrorKind::UnauthorizedClusterAccess => axum::http::StatusCode::UNAUTHORIZED,
                ErrorKind::ClusterNotFound => axum::http::StatusCode::NOT_FOUND,
                ErrorKind::MicrodeviceNotFound => axum::http::StatusCode::NOT_FOUND,
            })
            .body(self.message.into())
            .unwrap()
    }
}
