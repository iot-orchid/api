use axum::extract::State;
#[allow(unused_imports)]
use axum::{
    extract::{Json as ExtractJson, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};
use migration::Mode;
use sea_orm::{
    sea_query::table, DatabaseConnection, DbBackend, EntityTrait, QueryFilter, QueryTrait,
};
use serde::{Deserialize, Serialize};

use config::{Config, File, FileFormat};
use serde_json::json;
use std::{collections::HashMap, ptr::null, sync::Arc};

use entity::microdevice::Entity as Microdevice;
use entity::{cluster, microdevice};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid;

use sea_orm::entity::prelude::*;

#[allow(unused_imports)]
use sea_orm::{Database, DbErr};

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
    id: Option<String>,
    status: Option<DeviceStatus>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
struct DeviceCreate {
    id: String,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    kind: String,
    message: String,
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
        .apply_if(query.id, |q, v| {
            let microdevice_id: i32 = v.parse().unwrap();
            q.filter(microdevice::Column::Id.eq(microdevice_id))
        })
        .into_json()
        .all(&state.db)
        .await
        .unwrap();

    Ok((axum::http::StatusCode::OK, Json(res)))
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
        (status = 404),
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
        .apply_if(query.id, |q, v| {
            q.filter(microdevice::Column::Id.eq(v.parse::<i32>().unwrap()))
        })
        .exec(&state.db))
    .await
    {
        Ok(res) => Ok((AxumStatusCode::OK, "ok".to_string())),
        Err(e) => Err((
            AxumStatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                kind: "DB Error".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

use sea_orm::ActiveValue::Set;

#[utoipa::path(
    post,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    responses(
        (status = 200, body = [DeviceCreate]),
        (status = 404),
    ),
)]
async fn create_device(
    State(state): State<AppState>,
    Path(cluster_id): Path<String>,
    Query(_query): Query<DeviceQuery>,
    ExtractJson(data): Json<DeviceCreate>,
) -> String {
    let decoded_cluster_id = match URL_SAFE.decode(&cluster_id) {
        Ok(v) => v,
        Err(_) => return "Failed to decode base64 uuid from path".to_string(),
    };

    if decoded_cluster_id.len() != 16 {
        return "Malformed cluster ID".to_string();
    }

    let uuid = match Uuid::from_slice(&decoded_cluster_id) {
        Ok(uuid) => uuid,
        Err(_) => return "Could not convert path to uuid".to_string(),
    };

    let res = match (microdevice::ActiveModel {
        id: Set(data.id.parse().unwrap()),
        cluster_id: Set(uuid),
        name: Set(data.name),
        description: Set(data.description),
    }
    .insert(&state.db))
    .await
    {
        Ok(v) => v,
        Err(_) => return "DATABASE ERROR".to_string(),
    };

    res.cluster_id.into()
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
        (status = 404),
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
        Err(_) => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                kind: "test".to_string(),
                message: "test".to_string(),
            }),
        )),
    }
}

#[utoipa::path(
    get,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [String]),
        (status = 404),
    ),
)]
async fn list_clusters(
    State(state): State<AppState>,
) -> Result<
    (axum::http::StatusCode, Json<Vec<serde_json::Value>>),
    (axum::http::StatusCode, Json<ErrorResponse>),
> {
    match (cluster::Entity::find().all(&state.db)).await {
        Ok(v) => {
            for ele in v {
                println!("{}", URL_SAFE.encode(&ele.id))
            }

            Ok((
                axum::http::StatusCode::OK,
                Json(vec![json!({
                    "test":"test"
                })]),
            ))
        }
        Err(_) => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                kind: "database error".to_string(),
                message: "test".to_string(),
            }),
        )),
    }
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

use axum::http::status::StatusCode as AxumStatusCode;

fn decode_uuid(s: String) -> Result<uuid::Uuid, (AxumStatusCode, Json<ErrorResponse>)> {
    let decoded_str = match URL_SAFE.decode(s) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                AxumStatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    kind: "Decode Error".to_string(),
                    message: "Failed to decode supplied string".to_string(),
                }),
            ))
        }
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
        Err(_) => {
            return Err((
                AxumStatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    kind: "Decode Error".to_string(),
                    message: "Failed to decode supplied string".to_string(),
                }),
            ))
        }
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
        delete_device
    ),
    components(
        schemas (
            ClusterCreate,
            DeviceCreate,
            DeviceQuery,
            DeviceStatus,
        )
    ),
    tags(
        (name = "Clusters", description = "Cluster operations"),
        (name = "Microdevices", description = "Microdevice operations")
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
