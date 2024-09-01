use super::error::{Error, Result};
use crate::model::AppState;
use axum::extract::{Json as ExtractJson, State};
use axum::Json;
use entity::user;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UserCredentials {
    #[schema(example = "foo")]
    username: String,
    #[schema(example = "bar")]
    password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginSuccess {
    access_token: String,
    refresh_token: String,
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "Authentication",
    responses(
        (status = 200, body = [LoginSuccess]),
        (status = 404, body = [ErrorResponse]),
    ),
)]
pub async fn handler(
    State(state): State<AppState>,
    ExtractJson(payload): Json<UserCredentials>,
) -> Result<Json<LoginSuccess>> {
    let user = match user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await?
    {
        Some(user) => user,
        None => return Err(Error::UsernameNotFound),
    };

    if !bcrypt::verify(payload.password, &user.password_hash)? {
        return Err(Error::IncorrectPassword);
    }

    // let access_token_claims = Claims {
    //     sub: user.id.to_string(),
    //     exp: (chrono::Utc::now() + std::time::Duration::from_secs(60 * 60 * 5)).timestamp()
    //         as usize,
    //     iat: chrono::Utc::now().timestamp() as usize,
    // };

    // let refresh_token_claims = Claims {
    //     sub: user.id.to_string(),
    //     exp: (chrono::Utc::now() + std::time::Duration::from_secs(60 * 60 * 24)).timestamp()
    //         as usize,
    //     iat: chrono::Utc::now().timestamp() as usize,
    // };

    // let access_token = match jwt::encode(
    //     &jwt::Header::default(),
    //     &access_token_claims,
    //     &EncodingKey::from_secret("secret".as_bytes()),
    // ) {
    //     Ok(token) => token,
    //     Err(e) => return Err((AxumStatusCode::INTERNAL_SERVER_ERROR, Json(e.into()))),
    // };

    // let refresh_token = match jwt::encode(
    //     &jwt::Header::default(),
    //     &refresh_token_claims,
    //     &EncodingKey::from_secret("secret".as_bytes()),
    // ) {
    //     Ok(token) => token,
    //     Err(e) => return Err((AxumStatusCode::INTERNAL_SERVER_ERROR, Json(e.into()))),
    // };

    todo!()
}
