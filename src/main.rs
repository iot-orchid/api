#[allow(unused_imports)]
use axum::{
    extract::{Json as ExtractJson, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use config::{Config, File, FileFormat};
use std::sync::Arc;

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

#[allow(dead_code)]
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
#[derive(Deserialize, Debug)]
struct DeviceCreate {
    id: String,
}

async fn get_devices(Path(id): Path<String>, Query(query): Query<DeviceQuery>) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or(Status::Unknown)
    )
}

async fn delete_device(Path(id): Path<String>, Query(query): Query<DeviceQuery>) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or(Status::Unknown)
    )
}

async fn create_device(
    Path(id): Path<String>,
    Query(query): Query<DeviceQuery>,
    ExtractJson(_payload): Json<DeviceCreate>,
) -> String {
    format!(
        "Cluster ID: {}, Device ID: {}, Status: {}",
        id,
        query.id.unwrap_or("None".to_string()),
        query.status.unwrap_or(Status::Unknown)
    )
}

struct AppState {
    db: Database,
}

#[derive(Debug)]
struct Uri {
    db : String,
    ampq : String
}

#[tokio::main]
async fn main() {
    let builder = Config::builder()
        .add_source(File::new("config/settings_dev", FileFormat::Yaml));

    let uri = match builder.build() {
        Ok(config) => {
            if let Ok(tbl) = config.cache.into_table().as_ref() {

                let mut uri = Uri {
                    db: "".to_string(),
                    ampq: "".to_string(),
                };

                for (_, value) in tbl.iter() {
                    for (k, v) in match value.clone().into_table() {
                        Ok(tbl) => tbl,
                        Err(_) => continue,
                    } {
                        match k.as_str() {
                            "db" => uri.db = v.to_string(),
                            "ampq" => uri.ampq = v.to_string(),
                            _ => continue,
                        }
                    }
                }
            
                uri
            } else {
                eprintln!("Failed to load configuration: {:?}", "No table found");
                return;
            }
        }
        Err(err) => {
            eprintln!("Failed to load configuration: {:?}", err);
            return;
        }
    };

    println!("URI: {:?}", uri);

    // let db = match Database::connect("postgres://sea:sea@localhost:5432/sea").await {
    //     Ok(db) => db,
    //     Err(err) => {
    //         eprintln!("Failed to connect to database: {:?}", err);
    //         return;
    //     }
    // };

    let app = Router::new()
        .route("/clusters/:id/devices", get(get_devices))
        .route("/clusters/:id/devices", post(create_device))
        .route("/clusters/:id/devices", delete(delete_device));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
