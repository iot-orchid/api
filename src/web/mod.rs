use crate::model::ModelManager;
pub mod cluster;
pub mod error;
mod guard;
pub mod microdevice;
pub mod rpc;
pub mod session;
pub mod user;

#[allow(unused_imports)]
use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::AllowOrigin;

pub fn app(model_manager: ModelManager) -> Router {
    Router::new()
        .route("/clusters", post(cluster::create))
        .route("/clusters", get(cluster::get))
        .route("/clusters", delete(cluster::delete))
        .route(
            "/cluster/:clusterId/device/:microdeviceId",
            put(microdevice::update_device),
        )
        .route(
            "/cluster/:clusterId/devices/actions",
            post(rpc::rpc_handler),
        )
        .route("/cluster/:clusterId/devices", get(microdevice::get_devices))
        .route(
            "/cluster/:clusterId/devices",
            post(microdevice::create_device),
        )
        .route(
            "/cluster/:clusterId/devices",
            delete(microdevice::delete_device),
        )
        .route("/logout", post(session::logout))
        .route("/status", get(session::status))
        .layer(axum::middleware::from_fn(guard::jwt_guard))
        .route("/login", post(session::login))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(AllowOrigin::list(vec![
                    "http://localhost:3000".parse().unwrap(),
                    "http://localhost:3001".parse().unwrap(),
                    "http://localhost:5173".parse().unwrap(),
                    "https://iot-orchid.app".parse().unwrap(),
                    "https://www.iot-orchid.app".parse().unwrap(),
                ]))
                .allow_credentials(true)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::DELETE,
                    axum::http::Method::PUT,
                ])
                .allow_headers(vec![
                    "content-type".parse().unwrap(),
                    "authorization".parse().unwrap(),
                ]),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(model_manager)
}
