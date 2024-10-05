use axum::Router;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
mod auth;
mod config;
mod context;
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
        web::microdevice::add_topic,
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
            web::microdevice::DeviceQuery,
            web::microdevice::DeviceStatus,
            web::session::UserCredentials,
            web::session::LoginSuccess,
            web::rpc::JrpcExample,
            web::rpc::actions::MicrodeviceActions,
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
    let model_manager = ModelManager::new();

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
