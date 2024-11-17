use axum::Router;
use tracing_subscriber::EnvFilter;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
mod auth;
mod config;
mod context;
mod events;
mod model;
mod web;
use model::ModelManager;

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        web::cluster::create,
        web::cluster::get,
        web::microdevice::get_devices,
        web::microdevice::create_device,
        web::microdevice::delete_device,
        web::microdevice::update_device,
        web::session::login,
        web::session::status,
        web::session::logout,
        web::rpc::rpc_handler,
    ),
    components(
        schemas (
            model::cluster::ClusterCreate,
            model::cluster::ClusterRecord,
            model::microdevice::MicrodeviceCreate,
            model::microdevice::MicrodeviceGetParams,
            model::microdevice::MicrodeviceUpdateParams,
            model::microdevice::DeviceStatus,
            web::session::UserCredentials,
            web::session::LoginSuccess,
            web::rpc::JrpcExample,
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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let model_manager = ModelManager::new().await;
    let mm_api_ref = model_manager.clone();
    let mm_event_ref = model_manager.clone();

    let event_manager = events::EventManager::new(mm_event_ref);

    tokio::spawn(async move {
        let _ = event_manager.start().await;
    });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", web::app(mm_api_ref));

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
