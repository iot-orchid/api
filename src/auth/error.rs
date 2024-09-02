pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    JwtEncodeError(jsonwebtoken::errors::Error),
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Error::JwtEncodeError(e)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::JwtEncodeError(e) => write!(f, "Jwt encode error: {}", e),
        }
    }
}
