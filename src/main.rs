use axum::extract::State;
use axum::http::status::StatusCode as AxumStatusCode;
use axum::{
    extract::{Json as ExtractJson, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use bcrypt;
use config::{Config, File, FileFormat};
use entity::{cluster, microdevice, user, user_cluster};
use jsonwebtoken::{self as jwt, EncodingKey};
use sea_orm::entity::prelude::*;
use sea_orm::sqlx::types::chrono;
use sea_orm::ActiveValue::Set;
use sea_orm::Database;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid;

#[derive(Debug, Deserialize, ToSchema)]
enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Device {
    id: &'static str,
    status: DeviceStatus,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "online"),
            DeviceStatus::Offline => write!(f, "offline"),
            DeviceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl Serialize for DeviceStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[allow(dead_code)]
#[derive(Deserialize, ToSchema)]
struct DeviceQuery {
    id: Option<i32>,
    status: Option<DeviceStatus>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
struct DeviceCreate {
    name: String,
    description: String,
}

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

#[utoipa::path(
    get,
    path = "/clusters/{clusterID}/devices",
    tag = "Microdevices",
    params(
        ("clusterID" = String, Path, description="Cluster ID a existing cluster"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
        ("status" = Option<DeviceStatus>, Query, description="Microdevice Status Code"),
    ),
    responses(
        (status = 200, body = [String]),
        (status = 404),
    )
)]
async fn get_devices(
    State(state): State<AppState>,
    Path(cluster_id): Path<String>,
    Query(query): Query<DeviceQuery>,
) -> Result<
    (axum::http::StatusCode, Json<Vec<serde_json::Value>>),
    (axum::http::StatusCode, Json<ErrorResponse>),
> {
    let uuid = decode_uuid(cluster_id)?;

    let res = microdevice::Entity::find()
        .filter(microdevice::Column::ClusterId.eq(uuid))
        .apply_if(query.id, |q, v| q.filter(microdevice::Column::Id.eq(v)))
        .into_json()
        .all(&state.db)
        .await;

    match res {
        Err(e) => return Err((axum::http::StatusCode::BAD_REQUEST, Json(e.into()))),
        Ok(v) => Ok((axum::http::StatusCode::OK, Json(v))),
    }
}

#[utoipa::path(
    delete,
    path = "/clusters/{clusterID}/devices",
    tag = "Microdevices",
    params(
        ("clusterID" = String, Path, description="Cluster ID a existing cluster"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
        ("status" = Option<DeviceStatus>, Query, description="Microdevice Status Code"),
    ),
    responses(
        (status = 200, body = [String]),
        (status = 404, body = [ErrorResponse]),
    ),
)]
async fn delete_device(
    State(state): State<AppState>,
    Path(cluster_id): Path<String>,
    Query(query): Query<DeviceQuery>,
) -> Result<(AxumStatusCode, String), (AxumStatusCode, Json<ErrorResponse>)> {
    let uuid = decode_uuid(cluster_id)?;

    match (microdevice::Entity::delete_many()
        .filter(microdevice::Column::ClusterId.eq(uuid))
        .apply_if(query.id, |q, v| q.filter(microdevice::Column::Id.eq(v)))
        .exec(&state.db))
    .await
    {
        Ok(_) => Ok((AxumStatusCode::OK, "ok".to_string())),
        Err(e) => Err((AxumStatusCode::BAD_REQUEST, Json(e.into()))),
    }
}

#[utoipa::path(
    post,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    responses(
        (status = 200, body = [DeviceCreate]),
        (status = 404, body = [ErrorResponse]),
    ),
)]
async fn create_device(
    State(state): State<AppState>,
    Path(cluster_id): Path<String>,
    ExtractJson(data): Json<DeviceCreate>,
) -> Result<(AxumStatusCode, String), (AxumStatusCode, Json<ErrorResponse>)> {
    let uuid = decode_uuid(cluster_id)?;

    let _res = match (microdevice::ActiveModel {
        cluster_id: Set(uuid),
        name: Set(data.name),
        description: Set(data.description),
        ..Default::default()
    }
    .insert(&state.db))
    .await
    {
        Ok(v) => v,
        Err(e) => return Err((AxumStatusCode::BAD_REQUEST, Json(e.into()))),
    };

    Ok((AxumStatusCode::OK, "ok".to_string()))
}

#[derive(Deserialize, Serialize, ToSchema)]
struct ClusterCreate {
    name: String,
    region: String,
    description: String,
}

#[utoipa::path(
    post,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [ClusterCreate]),
        (status = 404, body = [ErrorResponse]),
    ),
)]
async fn create_cluster(
    State(state): State<AppState>,
    ExtractJson(data): Json<ClusterCreate>,
) -> Result<
    (axum::http::StatusCode, Json<ClusterCreate>),
    (axum::http::StatusCode, Json<ErrorResponse>),
> {
    let cluster_id = uuid::Uuid::new_v4();

    // TODO add check to prevent duplicate clusters based on the name
    // can either be done in the entity by making the NAME column unique
    // or by filtering and making sure no other entry exists with the same name.

    match (cluster::ActiveModel {
        id: Set(cluster_id),
        name: Set(data.name.clone()),
        ..Default::default()
    })
    .insert(&state.db)
    .await
    {
        Ok(_) => Ok((axum::http::StatusCode::OK, Json(data))),
        Err(e) => Err((axum::http::StatusCode::BAD_REQUEST, Json(e.into()))),
    }
}

#[derive(Serialize, ToSchema)]
struct ListClusterElement {
    uuid: String,
    name: String,
    encoded_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [ListClusterElement]),
        (status = 404, body = [ErrorResponse]),
    ),
)]
async fn list_clusters(
    State(state): State<AppState>,
) -> Result<(AxumStatusCode, Json<Vec<ListClusterElement>>), (AxumStatusCode, Json<ErrorResponse>)>
{
    let res = match (cluster::Entity::find().all(&state.db)).await {
        Ok(v) => v,
        Err(e) => return Err((AxumStatusCode::BAD_REQUEST, Json(e.into()))),
    };

    let elems = res
        .iter()
        .map(|c| ListClusterElement {
            uuid: c.id.to_string(),
            name: c.name.clone(),
            encoded_id: Some(URL_SAFE.encode(&c.id.as_bytes())),
        })
        .collect();

    Ok((AxumStatusCode::OK, Json(elems)))
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
        sub: user.username.clone(),
        exp: (chrono::Utc::now() + std::time::Duration::from_secs(60 * 15)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    let refresh_token_claims = Claims {
        sub: user.username.clone(),
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

fn decode_uuid(s: String) -> Result<uuid::Uuid, (AxumStatusCode, Json<ErrorResponse>)> {
    let decoded_str = match URL_SAFE.decode(s) {
        Ok(bytes) => bytes,
        Err(e) => return Err((AxumStatusCode::BAD_REQUEST, Json(e.into()))),
    };

    if decoded_str.len() != 16 {
        return Err((
            AxumStatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                kind: "Invalid String".to_string(),
                message: "String must be 16 bytes".to_string(),
            }),
        ));
    }

    match uuid::Uuid::from_slice(&decoded_str) {
        Ok(uuid) => Ok(uuid),
        Err(e) => return Err((AxumStatusCode::BAD_REQUEST, Json(e.into()))),
    }
}

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_cluster,
        list_clusters,
        get_devices,
        create_device,
        delete_device,
        handle_login,
    ),
    components(
        schemas (
            ClusterCreate,
            DeviceCreate,
            DeviceQuery,
            DeviceStatus,
            ListClusterElement,
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
    )
)]
struct ApiDoc;

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
        .route("/clusters", post(create_cluster))
        .route("/clusters", get(list_clusters))
        .route("/clusters/:clusterId/devices", get(get_devices))
        .route("/clusters/:clusterId/devices", post(create_device))
        .route("/clusters/:clusterId/devices", delete(delete_device))
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
