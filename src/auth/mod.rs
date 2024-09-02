pub mod error;

pub mod jwt_auth {
    use super::error::Result;
    use crate::config::CONFIG;
    use chrono::Utc;
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    pub fn encode(sub: String) -> Result<String> {
        let claims = Claims {
            sub,
            exp: (Utc::now() + Duration::from_secs(CONFIG.jwt.expires_in)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            iss: CONFIG.jwt.issuer.clone(),
        };

        let jwt_secret = CONFIG.jwt.secret.clone();
        let key = EncodingKey::from_secret(jwt_secret.as_bytes());

        Ok(jsonwebtoken::encode(&Header::default(), &claims, &key)?)
    }

    pub fn decode(token: &str) -> Result<Claims> {
        let jwt_secret = CONFIG.jwt.secret.clone();
        let key = DecodingKey::from_secret(jwt_secret.as_bytes());

        let token_data = jsonwebtoken::decode::<Claims>(token, &key, &Validation::default())?;
        Ok(token_data.claims)
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        pub sub: String,
        pub exp: usize,
        pub iat: usize,
        pub iss: String,
    }
}
