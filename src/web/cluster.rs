#[allow(unused_imports)]
use super::error::{Error, Result};
use crate::context::Ctx;
use crate::model::ModelManager;
#[allow(unused_imports)]
use axum::{
    extract::{Extension, Json as ExtractJson, Query, State},
    response::Json,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use entity::user;
use entity::{cluster, user_cluster};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::{entity::prelude::*, QueryTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    post,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [ClusterRecord]),
        (status = 401),
        (status = 400),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn create(
    State(state): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    ExtractJson(data): Json<ClusterCreate>,
) -> Result<Json<ClusterRecord>> {
    let cluster_uuid = uuid::Uuid::new_v4();
    let user_uuid = uuid::Uuid::parse_str(&ctx.uuid)?;

    let cluster = (cluster::ActiveModel {
        id: Set(cluster_uuid),
        name: Set(data.name.clone()),
        ..Default::default()
    })
    .insert(&state.db)
    .await?;

    let user_cluster = (user_cluster::ActiveModel {
        user_id: Set(user_uuid),
        cluster_id: Set(cluster_uuid),
    })
    .insert(&state.db)
    .await?;

    Ok(Json(ClusterRecord {
        encoded_uuid: Some(URL_SAFE.encode(user_cluster.cluster_id)),
        name: (cluster.name),
    }))
}

#[utoipa::path(
    get,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [ClusterRecord]),
        (status = 401),
        (status = 400),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn get(
    State(state): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    Query(params): Query<ClusterQuery>,
) -> Result<Json<Vec<ClusterRecord>>> {
    let user_uuid = uuid::Uuid::parse_str(&ctx.uuid)?;

    let query_uuid = match params.uuid {
        Some(uuid) => Some(parse_uuid(uuid)?),
        None => None,
    };

    let clusters = user::Entity::find_by_id(user_uuid)
        .find_also_related(cluster::Entity)
        .apply_if(query_uuid, |q, v| q.filter(cluster::Column::Id.eq(v)))
        .all(&state.db)
        .await?;

    let records: Vec<ClusterRecord> = clusters
        .into_iter()
        .filter_map(|(_, c)| {
            c.map(|sc| ClusterRecord {
                name: sc.name.clone(),
                encoded_uuid: Some(URL_SAFE.encode(sc.id)),
            })
        })
        .collect();

    Ok(Json(records))
}

fn parse_uuid(uuid_str: String) -> Result<Uuid> {
    match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            let decoded_str = URL_SAFE.decode(uuid_str)?;
            Ok(Uuid::from_slice(&decoded_str)?)
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct ClusterCreate {
    #[schema(example = "factory-a")]
    name: String,
    #[schema(example = "us-west-1")]
    region: String,
    #[schema(example = "Cluster of sensors in factory-a used for telemetry.")]
    description: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ClusterQuery {
    uuid: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ClusterRecord {
    #[schema(example = "<base64 encoded cluster uuid>")]
    encoded_uuid: Option<String>,
    #[schema(example = "factory-a")]
    name: String,
}
