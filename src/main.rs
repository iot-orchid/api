use axum::extract::State;
use axum::http::status::StatusCode as AxumStatusCode;
use axum::{
    extract::Json as ExtractJson,
    routing::{delete, get, post},
    Json, Router,
};
use bcrypt;
use config::{Config, File, FileFormat};
#[allow(unused_imports)]
use entity::{microdevice, user, user_cluster};
use jsonwebtoken::{self as jwt, DecodingKey, EncodingKey};
use sea_orm::entity::prelude::*;
use sea_orm::sqlx::types::chrono;
use sea_orm::Database;
use sea_orm::{EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid;

mod auth;
mod context;
mod model;
mod web;

use context::Ctx;
use model::AppState;

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    kind: String,
    message: String,
}

impl From<sea_orm::DbErr> for ErrorResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        ErrorResponse {
            kind: "Database Error".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<base64::DecodeError> for ErrorResponse {
    fn from(e: base64::DecodeError) -> Self {
        ErrorResponse {
            kind: "Decode Error".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<uuid::Error> for ErrorResponse {
    fn from(e: uuid::Error) -> Self {
        ErrorResponse {
            kind: "UUID Error".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<bcrypt::BcryptError> for ErrorResponse {
    fn from(e: bcrypt::BcryptError) -> Self {
        ErrorResponse {
            kind: "Authentication Error".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<jwt::errors::Error> for ErrorResponse {
    fn from(e: jwt::errors::Error) -> Self {
        ErrorResponse {
            kind: "JWT Error".to_string(),
            message: e.to_string(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
struct UserCredentials {
    #[schema(example = "foo")]
    username: String,
    #[schema(example = "bar")]
    password: String,
}

#[derive(Serialize, ToSchema)]
struct LoginSuccess {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
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
async fn handle_login(
    State(state): State<AppState>,
    ExtractJson(payload): Json<UserCredentials>,
) -> Result<(AxumStatusCode, Json<LoginSuccess>), (AxumStatusCode, Json<ErrorResponse>)> {
    let user = match (user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&state.db))
    .await
    {
        Ok(v) => {
            if let Some(u) = v {
                u
            } else {
                return Err((
                    AxumStatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        kind: "Authentication Error".to_string(),
                        message: "Invalid username or password".to_string(),
                    }),
                ));
            }
        }
        Err(e) => return Err((AxumStatusCode::INTERNAL_SERVER_ERROR, Json(e.into()))),
    };

    match bcrypt::verify(payload.password, &user.password_hash) {
        Ok(password_matches) if password_matches => (),
        Ok(_) => {
            return Err((
                AxumStatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    kind: "Authentication Error".to_string(),
                    message: "Invalid username or password".to_string(),
                }),
            ))
        }
        Err(e) => return Err((AxumStatusCode::UNAUTHORIZED, Json(e.into()))),
    }

    let access_token_claims = Claims {
        sub: user.id.to_string(),
        exp: (chrono::Utc::now() + std::time::Duration::from_secs(60 * 15)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    let refresh_token_claims = Claims {
        sub: user.id.to_string(),
        exp: (chrono::Utc::now() + std::time::Duration::from_secs(60 * 60 * 24)).timestamp()
            as usize,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    let access_token = match jwt::encode(
        &jwt::Header::default(),
        &access_token_claims,
        &EncodingKey::from_secret("secret".as_bytes()),
    ) {
        Ok(token) => token,
        Err(e) => return Err((AxumStatusCode::INTERNAL_SERVER_ERROR, Json(e.into()))),
    };

    let refresh_token = match jwt::encode(
        &jwt::Header::default(),
        &refresh_token_claims,
        &EncodingKey::from_secret("secret".as_bytes()),
    ) {
        Ok(token) => token,
        Err(e) => return Err((AxumStatusCode::INTERNAL_SERVER_ERROR, Json(e.into()))),
    };

    Ok((
        AxumStatusCode::OK,
        Json(LoginSuccess {
            access_token: access_token,
            refresh_token: refresh_token,
        }),
    ))
}

#[derive(Debug)]
struct AppConfig {
    db: String,
    _ampq: String,
    port: String,
    address: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            db: "postgres://sea:sea@localhost:5432/sea".to_string(),
            _ampq: "amqp://guest:guest@localhost:5672/%2f".to_string(),
            port: "3000".to_string(),
            address: "localhost".to_string(),
        }
    }
}

fn get_value<'a>(key: &'static str, map: &'a HashMap<String, config::Value>) -> &'a config::Value {
    match map.get(key) {
        Some(val) => val,
        None => panic!("{key} is not defined in the config YAML"),
    }
}

async fn jwt_guard(
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

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        web::cluster::create,
        web::cluster::get,
        web::microdevice::get_devices,
        web::microdevice::create_device,
        web::microdevice::delete_device,
        handle_login,
    ),
    components(
        schemas (
            web::cluster::ClusterCreate,
            web::cluster::ListClusterElement,
            web::microdevice::DeviceCreate,
            web::microdevice::DeviceQuery,
            web::microdevice::DeviceStatus,
            ErrorResponse,
            UserCredentials,
            LoginSuccess,
        )
    ),
    tags(
        (name = "Clusters", description = "Cluster operations"),
        (name = "Microdevices", description = "Microdevice operations"),
        (name = "Authentication", description = "Authentication operations"),
    ),
    servers(
        (url = "/api/v1", description = "API v1 base path")
    ),
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-ACCESS-TOKEN"))),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    let builder = Config::builder().add_source(File::new("config/settings_dev", FileFormat::Yaml));

    let cfg = match builder.build() {
        Ok(config) => match config.cache.into_table().as_ref() {
            Ok(tbl) => AppConfig {
                db: get_value("db", tbl).to_string(),
                _ampq: get_value("ampq", tbl).to_string(),
                port: get_value("port", tbl).to_string(),
                address: get_value("address", tbl).to_string(),
            },
            Err(err) => {
                panic!("failed to read configuration file\n{:?}", err)
            }
        },
        Err(err) => {
            panic!("failed to read configuration file\n{:?}", err)
        }
    };

    let db = match Database::connect(cfg.db).await {
        Ok(db) => {
            println!("Connected to database");
            db
        }
        Err(err) => {
            eprintln!("Failed to connect to database: {:?}", err);
            return;
        }
    };

    let state = AppState { db: db };

    let orchid = Router::new()
        .route("/clusters", post(web::cluster::create))
        .route("/clusters", get(web::cluster::get))
        .route(
            "/clusters/:clusterId/devices",
            get(web::microdevice::get_devices),
        )
        .route(
            "/clusters/:clusterId/devices",
            post(web::microdevice::create_device),
        )
        .route(
            "/clusters/:clusterId/devices",
            delete(web::microdevice::delete_device),
        )
        .layer(axum::middleware::from_fn(jwt_guard))
        .route("/login", post(handle_login))
        .with_state(state);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", orchid);

    if let Ok(listener) =
        tokio::net::TcpListener::bind(format!("{}:{}", cfg.address, cfg.port)).await
    {
        axum::serve(listener, app).await.unwrap();
    } else {
        panic!("failed to bind to address.")
    }
}
