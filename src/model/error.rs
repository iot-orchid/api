pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DatabaseError(sea_orm::error::DbErr),
    UuidError(uuid::Error),
    Base64DecodeError(base64::DecodeError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Base64DecodeError(e) => write!(f, "Base64 decode error: {}", e),
            Error::DatabaseError(e) => write!(f, "Database error: {}", e),
            Error::UuidError(e) => write!(f, "Uuid error: {}", e),
        }
    }
}

impl From<sea_orm::error::DbErr> for Error {
    fn from(e: sea_orm::error::DbErr) -> Self {
        Error::DatabaseError(e)
    }
}

impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Self {
        Error::UuidError(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::Base64DecodeError(e)
    }
}
