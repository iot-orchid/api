use super::error::Result;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use uuid::Uuid;

pub fn parse_uuid(uuid_str: &String) -> Result<Uuid> {
    match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            let decoded_str = URL_SAFE.decode(uuid_str)?;
            Ok(Uuid::from_slice(&decoded_str)?)
        }
    }
}
