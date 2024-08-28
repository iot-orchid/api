use axum::extract::State;
#[allow(unused_imports)]
use axum::{
    extract::{Json as ExtractJson, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};
use sea_orm::{sea_query::table, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

use config::{Config, File, FileFormat};
use std::{collections::HashMap, ptr::null, sync::Arc};

use entity::cluster;
use entity::microdevice::Entity as Microdevice;

use serde_json;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[allow(unused_imports)]
use sea_orm::{Database, DbErr};

#[derive(Debug, Deserialize)]
enum Status {
    Online,
    Offline,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Device {
    id: &'static str,
    status: Status,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Online => write!(f, "Online"),
            Status::Offline => write!(f, "Offline"),
            Status::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Deserialize)]
struct DeviceQuery {
    id: Option<String>,
    status: Option<Status>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
struct DeviceCreate {
    id: String,
}

#[utoipa::path(
    get,
    path = "/clusters/{id}/devices",
    tag = "Microdevices",
    responses(
        (status = 200, body = [String]),
        (status = 404),
    )
)]
async fn get_devices(Path(id): Path<String>, Query(query): Query<DeviceQuery>) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or(Status::Unknown)
    )
}

#[utoipa::path(
    delete,
    path = "/clusters/{id}/devices",
    tag = "Microdevices",
    params(
        ("id" = String, Path, description="Cluster ID"),
    ),
    responses(
        (status = 200, body = [String]),
        (status = 404),
    ),
)]
async fn delete_device(Path(id): Path<String>, Query(query): Query<DeviceQuery>) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or(Status::Unknown)
    )
}

#[utoipa::path(
    post,
    path = "/clusters/{id}/devices",
    tag = "Microdevices",
    responses(
        (status = 200, body = [DeviceCreate]),
        (status = 404),
    ),
)]
async fn create_device(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<DeviceQuery>,
    ExtractJson(_payload): Json<DeviceCreate>,
) -> String {
    "Hello".to_string()
}

#[derive(Deserialize, Serialize, ToSchema)]
struct ClusterCreate {
    id: Option<String>,
    name: String,
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
) -> Json<ClusterCreate> {
    Json(ClusterCreate {
        id: Some("1".to_string()),
        name: data.name,
        description: data.description,
    })
}

#[derive(Debug)]
struct AppConfig {
    db: String,
    ampq: String,
    port: String,
    address: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            db: "postgres://sea:sea@localhost:5432/sea".to_string(),
            ampq: "amqp://guest:guest@localhost:5672/%2f".to_string(),
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

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_cluster,
        get_devices,
        create_device,
        delete_device
    ),
    components(
        schemas (
            ClusterCreate,
            DeviceCreate
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
                ampq: get_value("ampq", tbl).to_string(),
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
        .route("/clusters/:id/devices", get(get_devices))
        .route("/clusters/:id/devices", post(create_device))
        .route("/clusters/:id/devices", delete(delete_device))
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
