use crate::context::Ctx;
use crate::model::AppState;
use axum::{
    extract::{Extension, Json as ExtractJson, Path, Query, State},
    http::StatusCode as AxumStatusCode,
    response::{IntoResponse, Json},
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use entity::{cluster, microdevice, user, user_cluster};
use sea_orm::{entity::prelude::*, QueryTrait};
use sea_orm::{ActiveValue::Set, JoinType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::web::error::{Error, Result};

#[utoipa::path(
    get,
    path = "/clusters/{clusterID}/devices",
    tag = "Microdevices",
    params(
        ("clusterID" = String, Path, description="Cluster ID a existing cluster"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
        ("status" = Option<DeviceStatus>, Query, description="Microdevice Status Code"),
    ),
    responses(
        (status = 200, body = [String]),
        (status = 404),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn get_devices(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_id): Path<String>,
    Query(query): Query<DeviceQuery>,
) -> Result<Json<Vec<DeviceRecord>>> {
    todo!()
}

#[utoipa::path(
    delete,
    path = "/clusters/{clusterID}/devices",
    tag = "Microdevices",
    params(
        ("clusterID" = String, Path, description="Cluster ID a existing cluster"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
        ("status" = Option<DeviceStatus>, Query, description="Microdevice Status Code"),
    ),
    responses(
        (status = 200, body = [String]),
        (status = 404, body = [ErrorResponse]),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn delete_device(
    State(state): State<AppState>,
    Path(cluster_id): Path<String>,
    Query(query): Query<DeviceQuery>,
) -> impl IntoResponse {
    todo!()
}

#[utoipa::path(
    post,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    responses(
        (status = 200, body = [DeviceCreate]),
        (status = 404, body = [ErrorResponse]),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn create_device(
    State(state): State<AppState>,
    Path(cluster_id): Path<String>,
    ExtractJson(data): Json<DeviceCreate>,
) -> impl IntoResponse {
    todo!()
}

fn decode_uuid(s: String) -> Result<Uuid> {
    let decoded_str = URL_SAFE
        .decode(s.as_bytes())
        .map_err(|e| Error::DecodeError(e))?;

    let uuid = uuid::Uuid::from_slice(&decoded_str).map_err(|e| Error::InvalidUuid(e))?;

    Ok(uuid)
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "online"),
            DeviceStatus::Offline => write!(f, "offline"),
            DeviceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, ToSchema)]
pub struct DeviceQuery {
    id: Option<i32>,
    status: Option<DeviceStatus>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
pub struct DeviceCreate {
    name: String,
    description: String,
}

#[derive(Serialize, ToSchema)]
pub struct DeviceRecord {
    uuid: String,
    name: String,
}
