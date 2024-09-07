use super::error::{Error, Result};
use crate::auth;
use crate::model::ModelManager;
use axum::extract::State;
use axum::Json;
use axum_extra::extract::CookieJar;
use entity::user;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Handles the login request.
///
/// This function is responsible for authenticating the user by checking the provided credentials.
/// It receives the `ModelManager` state, `CookieJar`, and `Json<UserCredentials>` as input.
/// It returns a `Result` containing the updated `CookieJar` or an `Error` if authentication fails.
///
/// # Examples
///
/// ```rust
/// use axum::handler::post;
/// use axum::Router;
/// use iot_orchid::api::web::session::{login, logout};
///
/// let app = Router::new()
///     .route("/login", post(login))
///     .route("/logout", post(logout));
/// ```
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
) -> Result<CookieJar> {
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

    let access_cookie = auth::jwt_auth::gen_access_cookie(user.id.to_string())?;
    let refresh_token = auth::jwt_auth::gen_refresh_cookie(user.id.to_string())?;

    Ok(jar.add(access_cookie).add(refresh_token))
}

/// Handles the logout request.
///
/// This function is responsible for removing the access and refresh tokens from the `CookieJar`.
/// It receives the `CookieJar` as input and returns a `Result` containing the updated `CookieJar`
/// or an `Error` if the expected cookies are not found.
///
/// # Examples
///
/// ```rust
/// use axum::handler::post;
/// use axum::Router;
/// use iot_orchid::api::web::session::{login, logout};
///
/// let app = Router::new()
///     .route("/login", post(login))
///     .route("/logout", post(logout));
/// ```
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
pub async fn logout(jar: CookieJar) -> Result<CookieJar> {
    
    let access_cookie = auth::jwt_auth::nullify_access_cookie();
    let refresh_cookie = auth::jwt_auth::nullify_refresh_cookie();

    Ok(jar.add(access_cookie).add(refresh_cookie))
}

/// Checks if the user is authenticated.
///
/// This function checks if the user is authenticated by verifying the presence of the access token
/// in the `CookieJar`. It receives the `CookieJar` as input and returns a `Result` indicating
/// whether the user is authenticated or not.
///
/// # Examples
///
/// ```rust
/// use axum::handler::get;
/// use axum::Router;
/// use iot_orchid::api::web::session::{login, logout, refresh, status};
///
/// let app = Router::new()
///     .route("/login", post(login))
///     .route("/logout", post(logout))
///     .route("/refresh", post(refresh))
///     .route("/status", get(status));
/// ```
#[utoipa::path(
    get,
    path = "/status",
    tag = "Authentication",
    responses(
        (status = 200),
        (status = 401),
        (status = 400),
    ),
)]
pub async fn status() -> Result<&'static str> {
    Ok("Authenticated")
}

// pub async fn refresh(jar: CookieJar) -> Result<CookieJar> {
//     // Check if the refresh token cookie exists and is valid
//     let refresh_claims = match jar.get(REFRESH_TOKEN_COOKIE_NAME) {
//         Some(cookie) => {
//             let refresh_token = cookie.clone();
//             auth::jwt_auth::decode(refresh_token.value())?
//         }
//         None => return Err(Error::ExpectedCookiesNotFound),
//     };

//     // Get the access token for deletion
//     let access_cookie = match jar.get(ACCESS_TOKEN_COOKIE_NAME) {
//         Some(cookie) => cookie.clone(),
//         None => return Err(Error::ExpectedCookiesNotFound),
//     };

//     // Delete the access token
//     let jar = jar.remove(access_cookie);

//     // Get the user ID from the refresh token claims
//     let user_id = refresh_claims.sub;

//     // Create a new access token
//     let access_token = auth::jwt_auth::gen_access_cookie(user_id.clone())?;

//     // Update the access token cookie
//     let access_cookie = Cookie::build(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, access_token))
//         .http_only(true)
//         .build();

//     Ok(jar.add(access_cookie))
// }

/// Represents the user credentials for login.
///
/// This struct is used to deserialize the JSON payload containing the username and password
/// for the login request.
#[derive(Deserialize, ToSchema)]
pub struct UserCredentials {
    #[schema(example = "foo")]
    username: String,
    #[schema(example = "bar")]
    password: String,
}

/// Represents the login success response.
///
/// This struct is used to serialize the access and refresh tokens in the login success response.
#[derive(Serialize, ToSchema)]
pub struct LoginSuccess {
    access_token: String,
    refresh_token: String,
}
