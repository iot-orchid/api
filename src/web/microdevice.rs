use crate::context::Ctx;
use crate::model::AppState;
use axum::{
    extract::{Extension, Json as ExtractJson, Path, Query, State},
    response::Json,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use entity::{microdevice, user_cluster};
use sea_orm::{entity::prelude::*, QueryTrait, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::web::error::{Error, Result};

#[utoipa::path(
    get,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    params(
        ("clusterId" = String, Path, description="Cluster ID a existing cluster"),
        ("name" = Option<String>, Query, description="Microdevice Name"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
        ("status" = Option<DeviceStatus>, Query, description="Microdevice Status Code"),
    ),
    responses(
        (status = 200),
        (status = 401),
        (status = 400),

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
) -> Result<Json<Vec<Value>>> {
    let (_, cluster_uuid) = check_membership(ctx.uuid, cluster_id, &state).await?;

    let devices = microdevice::Entity::find()
        .filter(microdevice::Column::ClusterId.eq(cluster_uuid))
        .apply_if(query.id, |q, id| q.filter(microdevice::Column::Id.eq(id)))
        .apply_if(query.name, |q, name| {
            q.filter(microdevice::Column::Name.eq(name.to_lowercase()))
        })
        .into_json()
        .all(&state.db)
        .await?;

    Ok(Json(devices))
}

#[utoipa::path(
    delete,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    params(
        ("clusterId" = String, Path, description="Cluster ID a existing cluster"),
        ("name" = Option<String>, Query, description="Microdevice Name"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
        ("status" = Option<DeviceStatus>, Query, description="Microdevice Status Code"),
    ),
    responses(
        (status = 200),
        (status = 401),
        (status = 400),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn delete_device(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_id): Path<String>,
    Query(_query): Query<DeviceQuery>,
) -> Result<Json<Vec<Value>>> {
    let (user_uuid, cluster_uuid) = check_membership(ctx.uuid, cluster_id, &state).await?;

    // Check if the user is a member of the cluster
    let user_cluster = user_cluster::Entity::find_by_id((user_uuid, cluster_uuid))
        .all(&state.db)
        .await?;

    if user_cluster.is_empty() {
        return Err(Error::UnauthorizedClusterAccess);
    }

    todo!()
}

async fn check_membership<S>(user_id: S, cluster_id: S, state: &AppState) -> Result<(Uuid, Uuid)>
where
    S: Into<String>,
{
    let user_uuid = Uuid::parse_str(&user_id.into()).map_err(|e| Error::InvalidUuid(e))?;
    let cluster_uuid = decode_uuid(cluster_id.into())?;

    // Check if the user is a member of the cluster
    let user_cluster = user_cluster::Entity::find_by_id((user_uuid.clone(), cluster_uuid))
        .all(&state.db)
        .await?;

    if user_cluster.is_empty() {
        return Err(Error::UnauthorizedClusterAccess);
    }

    Ok((user_uuid, cluster_uuid))
}

#[utoipa::path(
    post,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    params(
        ("clusterId" = String, Path, description="Cluster ID a existing cluster"),
    ),
    responses(
        (status = 200),
        (status = 401),
        (status = 400),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn create_device(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_id): Path<String>,
    ExtractJson(data): Json<DeviceCreate>,
) -> Result<Json<DeviceCreate>> {
    let (_, cluster_uuid) = check_membership(ctx.uuid, cluster_id, &state).await?;

    microdevice::ActiveModel {
        name: Set(data.clone().name.to_lowercase()),
        description: Set(data.clone().description),
        cluster_id: Set(cluster_uuid),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    Ok(Json(data))
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
    name: Option<String>,
    id: Option<i32>,
    status: Option<DeviceStatus>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct DeviceCreate {
    name: String,
    description: String,
}

#[derive(Serialize, ToSchema)]
pub struct DeviceRecord {
    uuid: String,
    name: String,
}
