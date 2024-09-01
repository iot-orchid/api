use crate::context::Ctx;
use jsonwebtoken::{self as jwt, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

pub async fn jwt_guard(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let token = match request.headers().get("X-ACCESS-TOKEN") {
        Some(token) => token,
        None => {
            return axum::http::Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body("Unauthorized".into())
                .unwrap()
        }
    };

    let token = match token.to_str() {
        Ok(token) => token,
        Err(_) => {
            return axum::http::Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body("Unauthorized".into())
                .unwrap()
        }
    };

    let key = "secret".to_string();
    let key = DecodingKey::from_secret(key.as_bytes());

    let token = match jwt::decode::<Claims>(&token, &key, &jwt::Validation::default()) {
        Ok(token) => token,
        Err(_) => {
            return axum::http::Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body("Unauthorized".into())
                .unwrap()
        }
    };

    let ctx = Ctx {
        uuid: token.claims.sub.clone(),
    };

    match request.extensions_mut().insert(ctx) {
        Some(_) => {
            return axum::http::Response::builder()
                .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal Server Error".into())
                .unwrap()
        }
        None => (),
    }

    next.run(request).await
}
