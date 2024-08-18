#[allow(unused_imports)]
use axum::{
    extract::{Json as ExtractJson, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
enum Status {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug)]
struct Device {
    id: &'static str,
    status: Status,
}

#[derive(Debug)]
struct Cluster {
    id: &'static str,
    name: &'static str,
    devices: Vec<Device>,
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

#[derive(Deserialize)]
struct DeviceQuery {
    id: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct DeviceCreate {
    id: String,
}

async fn get_devices(Path(id): Path<String>, Query(query): Query<DeviceQuery>) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or("None".to_string())
    )
}

async fn delete_device(Path(id): Path<String>, Query(query): Query<DeviceQuery>) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or("None".to_string())
    )
}

async fn create_device(
    Path(id): Path<String>,
    Query(query): Query<DeviceQuery>,
    ExtractJson(payload): Json<DeviceCreate>,
) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}, Payload: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or("None".to_string()),
        payload.id
    )
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/clusters/:id/devices", get(get_devices))
        .route("/clusters/:id/devices", post(create_device))
        .route("/clusters/:id/devices", delete(delete_device));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
