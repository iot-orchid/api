use axum::Router;
use sea_orm::Database;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
mod auth;
mod config;
mod context;
mod model;
mod web;
use model::AppState;

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
            web::cluster::ClusterRecord,
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
    let db = match Database::connect(config::CONFIG.db.clone()).await {
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

    if let Ok(listener) = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config::CONFIG.address,
        config::CONFIG.port
    ))
    .await
    {
        axum::serve(listener, app).await.unwrap();
    } else {
        panic!("failed to bind to address.")
    }
}
