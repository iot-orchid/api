use crate::auth;
use crate::context::Ctx;

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

    let claims = match auth::jwt_auth::decode(token) {
        Ok(token) => token,
        Err(_) => {
            return axum::http::Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body("Unauthorized".into())
                .unwrap()
        }
    };

    let ctx = Ctx { uuid: claims.sub };

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
