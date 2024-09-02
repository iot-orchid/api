use crate::model::ModelManager;
pub mod cluster;
pub mod error;
mod guard;
pub mod login;
pub mod microdevice;
pub mod user;

#[allow(unused_imports)]
use axum::routing::{delete, get, post, put};
use axum::Router;

pub fn app(model_manager: ModelManager) -> Router {
    Router::new()
        .route("/clusters", post(cluster::create))
        .route("/clusters", get(cluster::get))
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
        .layer(axum::middleware::from_fn(guard::jwt_guard))
        .route("/login", post(login::handler))
        .with_state(model_manager)
}
