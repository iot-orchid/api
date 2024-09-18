use crate::model::ModelManager;
pub mod cluster;
pub mod error;
mod guard;
pub mod microdevice;
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
        .route("/clusters/:clusterId/devices", put(microdevice::add_topic))
        .route(
            "/clusters/:clusterId/devices",
            get(microdevice::get_devices),
        )
        .route(
            "/clusters/:clusterId/devices",
            post(microdevice::create_device),
        )
        .route(
            "/clusters/:clusterId/devices",
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
        .with_state(model_manager)
}
