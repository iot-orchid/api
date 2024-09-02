use super::error::{Error, Result};
use crate::auth;
use crate::model::AppState;
use axum::extract::{Json as ExtractJson, State};
use axum::Json;
use entity::user;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    post,
    path = "/login",
    tag = "Authentication",
    responses(
        (status = 200, body = [LoginSuccess]),
        (status = 401),
        (status = 400),
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

    let access_token = auth::jwt_auth::encode(user.id.to_string())?;
    let refresh_token = auth::jwt_auth::encode(user.id.to_string())?;

    Ok(Json(LoginSuccess {
        access_token,
        refresh_token,
    }))
}

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
