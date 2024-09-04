pub mod error;

/// Module for JWT authentication
pub mod jwt_auth {
    use super::error::Result;
    use crate::config::CONFIG;
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use chrono::Utc;
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    /// The name of the access token cookie.
    pub const ACCESS_TOKEN_COOKIE_NAME: &str = "iotorchid_access_token";

    /// The name of the refresh token cookie.
    pub const REFRESH_TOKEN_COOKIE_NAME: &str = "iotorchid_refresh_token";

    /// Configuration for the cookies
    const COOKIE_CFG_SAME_SITE: SameSite = SameSite::Lax;
    const COOKIE_CFG_HTTP_ONLY: bool = true;
    const COOKIE_CFG_SECURE: bool = false;
    const COOKIE_CFG_PATH: &str = "/";

    /// Generates a Cookie containing a JWT access token for the specified subject
    ///

    ///
    /// # Arguments
    ///
    /// * `sub` - The user base-64 encoded UUID
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated cookie if successful or an `Error` if the token generation fails
    pub fn gen_access_cookie(sub: String) -> Result<Cookie<'static>> {
        let claims = Claims {
            sub,
            exp: (Utc::now() + Duration::from_secs(CONFIG.jwt.access_expires_in)).timestamp()
                as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            iss: CONFIG.jwt.issuer.clone(),
        };

        let jwt_secret = CONFIG.jwt.secret.clone();
        let key = EncodingKey::from_secret(jwt_secret.as_bytes());

        // Encode the access token
        let access_token = jsonwebtoken::encode(&Header::default(), &claims, &key)?;

        let cookie = Cookie::build(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, access_token))
            .http_only(COOKIE_CFG_HTTP_ONLY)
            .same_site(COOKIE_CFG_SAME_SITE)
            .secure(COOKIE_CFG_SECURE)
            .path(COOKIE_CFG_PATH)
            .build();

        Ok(cookie)
    }

    /// Generates a Cookie containing a JWT refresh token for the specified subject
    ///
    /// # Arguments
    ///
    /// * `sub` - The subject of the token
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated cookie if successful or an `Error` if the token generation fails
    ///  
    pub fn gen_refresh_cookie(sub: String) -> Result<Cookie<'static>> {
        let claims = Claims {
            sub,
            exp: (Utc::now() + Duration::from_secs(CONFIG.jwt.refresh_expires_in)).timestamp()
                as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            iss: CONFIG.jwt.issuer.clone(),
        };

        let jwt_secret = CONFIG.jwt.secret.clone();
        let key = EncodingKey::from_secret(jwt_secret.as_bytes());

        let refresh_token = jsonwebtoken::encode(&Header::default(), &claims, &key)?;

        let cookie = Cookie::build(Cookie::new(REFRESH_TOKEN_COOKIE_NAME, refresh_token))
            .http_only(COOKIE_CFG_HTTP_ONLY)
            .same_site(COOKIE_CFG_SAME_SITE)
            .secure(COOKIE_CFG_SECURE)
            .path(COOKIE_CFG_PATH)
            .build();

        Ok(cookie)
    }

    /// Decodes the specified token and returns the claims
    ///
    /// # Arguments
    ///
    /// * `token` - The token to decode
    ///
    /// # Returns
    ///
    /// The decoded claims as a `Result` containing a `Claims` struct
    pub fn decode(token: &str) -> Result<Claims> {
        let jwt_secret = CONFIG.jwt.secret.clone();
        let key = DecodingKey::from_secret(jwt_secret.as_bytes());

        let token_data = jsonwebtoken::decode::<Claims>(token, &key, &Validation::default())?;
        Ok(token_data.claims)
    }

    /// Struct representing the claims of a JWT token
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        pub sub: String,
        pub exp: usize,
        pub iat: usize,
        pub iss: String,
    }
}
