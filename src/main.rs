use axum::Router;
use config::{Config, File, FileFormat};
use sea_orm::Database;
use std::collections::HashMap;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
mod auth;
mod context;
mod model;
mod web;
use model::AppState;

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

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        web::cluster::create,
        web::cluster::get,
        web::microdevice::get_devices,
        web::microdevice::create_device,
        web::microdevice::delete_device,
        web::login::handler,
    ),
    components(
        schemas (
            web::cluster::ClusterCreate,
            web::cluster::ListClusterElement,
            web::microdevice::DeviceCreate,
            web::microdevice::DeviceQuery,
            web::microdevice::DeviceStatus,
            web::login::UserCredentials,
            web::login::LoginSuccess,
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

    let model_manager = AppState { db };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", web::app(model_manager));

    if let Ok(listener) =
        tokio::net::TcpListener::bind(format!("{}:{}", cfg.address, cfg.port)).await
    {
        axum::serve(listener, app).await.unwrap();
    } else {
        panic!("failed to bind to address.")
    }
}
