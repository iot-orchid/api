use super::error::Result;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use uuid::Uuid;

/// Method to parse a cluster ID from a string into a UUID
/// 
/// This allows both base64 encoded and UUID strings to be used as cluster IDs
/// during API calls.
/// 
pub fn parse_cluster_id(uuid_str: &String) -> Result<Uuid> {
    
    match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            let decoded_str = URL_SAFE.decode(uuid_str)?;
            Ok(Uuid::from_slice(&decoded_str)?)
        }
    }
}

// Method to parse a microdevice ID from an integer
//
// This method is used to parse a microdevice ID from an integer. This is
// useful when the microdevice ID is passed as a parameter in an API call.
//
pub fn parse_microdevice_id(id: i32) -> Result<i32> {
    Ok(id)
}
