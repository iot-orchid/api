use super::error::Error;
use crate::auth;
use crate::model::ModelManager;
use axum::extract::State;
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
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
pub async fn login(
    State(state): State<ModelManager>,
    jar: CookieJar,
    Json(payload): Json<UserCredentials>,
) -> Result<CookieJar, Error> {
    println!("Login request received");

    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await?;

    let user = match user {
        Some(user) => user,
        None => return Err(Error::UsernameNotFound),
    };

    if !bcrypt::verify(payload.password, &user.password_hash)? {
        return Err(Error::IncorrectPassword);
    }

    let access_token = auth::jwt_auth::gen_access_token(user.id.to_string())?;
    let refresh_token = auth::jwt_auth::gen_refresh_token(user.id.to_string())?;

    println!("access_token: {}", access_token);
    println!("refresh_token: {}", refresh_token);

    let cookies = [
        Cookie::build(Cookie::new("iotorchid_access_jwt", access_token))
            .http_only(true)
            .build(),
        Cookie::build(Cookie::new("iotorchid_refresh_token", refresh_token))
            .http_only(true)
            .build(),
    ];

    Ok(jar.add(cookies[0].clone()).add(cookies[1].clone()))
}

#[utoipa::path(
    post,
    path = "/logout",
    tag = "Authentication",
    responses(
        (status = 200),
        (status = 401),
        (status = 400),
    ),
)]
pub async fn logout(jar: CookieJar) -> Result<CookieJar, Error> {
    let access_cookie = match jar.get("iotorchid_access_jwt") {
        Some(cookie) => cookie.clone(),
        None => return Err(Error::ExpectedCookiesNotFound),
    };

    let refresh_cookie = match jar.get("iotorchid_refresh_token") {
        Some(cookie) => cookie.clone(),
        None => return Err(Error::ExpectedCookiesNotFound),
    };

    Ok(jar.remove(access_cookie).remove(refresh_cookie))
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
