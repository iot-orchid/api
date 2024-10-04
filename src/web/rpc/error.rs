use axum_jrpc::error::{JsonRpcError, JsonRpcErrorReason};
use axum_jrpc::Value;

#[derive(Debug)]
pub enum Error {
    SerdeJson(serde_json::Error),
    InvalidMethod(String),
}

pub type Result<T> = std::result::Result<T, Error>;
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error")
    }
}
 
impl From<Error> for JsonRpcError{
    fn from(e: Error) -> Self {
        match e {
            Error::SerdeJson(e) => JsonRpcError::new(
                JsonRpcErrorReason::InvalidRequest,
                e.to_string(),
                Value::default(),
            ),

            Error::InvalidMethod(e) => JsonRpcError::new(
                JsonRpcErrorReason::MethodNotFound,
                e,
                Value::default(),
            )
        }
    }

}


impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e) 
    }
}
