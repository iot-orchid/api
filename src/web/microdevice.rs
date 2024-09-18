use super::error::{Error, Result};
use crate::context::Ctx;
use crate::model::ModelManager;
use axum::{
    extract::{Extension, Json as ExtractJson, Path, Query, State},
    response::Json,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use entity::{microdevice, user_cluster};
use futures::future;
use sea_orm::{entity::prelude::*, IntoActiveModel, QuerySelect, QueryTrait, SelectColumns, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    params(
        ("clusterId" = String, Path, description="Cluster ID a existing cluster"),
        ("name" = Option<String>, Query, description="Microdevice Name"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
        ("status" = Option<DeviceStatus>, Query, description="Microdevice Status Code"),
        ("include_topics" = Option<bool>, Query, description="Include Topics", example=true),
        ("include_description" = Option<bool>, Query, description="Include Description", example=true),
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
    State(state): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_id): Path<String>,
    Query(query): Query<DeviceQuery>,
) -> Result<Json<Vec<Value>>> {
    let (_, cluster_uuid) = check_membership(ctx.uuid, cluster_id, &state).await?;

    let devices = microdevice::Entity::find()
        .select_only()
        .select_column(microdevice::Column::Name)
        .select_column(microdevice::Column::Id)
        .apply_if(query.include_topics, |q, _| {
            q.select_column(microdevice::Column::Topics)
        })
        .apply_if(query.include_description, |q, _| {
            q.select_column(microdevice::Column::Description)
        })
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
    State(state): State<ModelManager>,
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

#[derive(Serialize, ToSchema)]
pub struct Topic {
    topic: String,
}

/// Add a topic to a microdevice
///
/// This endpoint allows you to add a topic to a microdevice. The topic will be used to filter messages that are sent to the microdevice.
///
#[utoipa::path(
    put,
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
pub async fn add_topic(
    State(state): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_id): Path<String>,
    ExtractJson(data): Json<Vec<String>>,
) -> Result<Json<Vec<String>>> {
    let (_, cluster_uuid) = check_membership(ctx.uuid, cluster_id, &state).await?;

    let mut microdevice = microdevice::Entity::find()
        .filter(microdevice::Column::ClusterId.eq(cluster_uuid))
        .all(&state.db)
        .await?;

    let md_futures: Vec<_> = microdevice
        .iter_mut()
        .map(|md| {
            let mut active_md: microdevice::ActiveModel = md.clone().into_active_model();

            if let Some(md_topics) = active_md.topics.clone().into_value() {
                if let Some(arr) = md_topics.as_ref_array() {
                    let mut new_topics = data.clone();
                    new_topics.extend(arr.iter().map(|v| v.to_string()));
                    active_md.topics = Set(Some(serde_json::json!(new_topics)));
                }
            }

            active_md.update(&state.db)
        })
        .collect();

    future::try_join_all(md_futures).await?;

    Ok(Json(data))
}

async fn check_membership<S>(
    user_id: S,
    cluster_id: S,
    state: &ModelManager,
) -> Result<(Uuid, Uuid)>
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
    State(state): State<ModelManager>,
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
#[derive(Deserialize, ToSchema, Debug)]
pub struct DeviceQuery {
    name: Option<String>,
    id: Option<i32>,
    status: Option<DeviceStatus>,
    #[schema(example = true)]
    include_topics: Option<bool>,
    #[schema(example = true)]
    include_description: Option<bool>,
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
