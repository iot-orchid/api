use crate::context::Ctx;
use crate::model::AppState;
use axum::{
    extract::{Extension, Json as ExtractJson, State},
    http::StatusCode as AxumStatusCode,
    response::{IntoResponse, Json},
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use entity::user;
use entity::{cluster, user_cluster};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    post,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [ClusterCreate]),
        (status = 404, body = [ErrorResponse]),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    ExtractJson(data): Json<ClusterCreate>,
) -> impl IntoResponse {
    let cluster_id = uuid::Uuid::new_v4();

    let uuid = match uuid::Uuid::parse_str(&ctx.uuid) {
        Ok(uuid) => uuid,
        Err(_) => return Err((AxumStatusCode::INTERNAL_SERVER_ERROR, ())),
    };

    let _res = (cluster::ActiveModel {
        id: Set(cluster_id),
        name: Set(data.name.clone()),
        ..Default::default()
    })
    .insert(&state.db)
    .await
    .unwrap();

    let _res = (user_cluster::ActiveModel {
        user_id: Set(uuid),
        cluster_id: Set(cluster_id),
    })
    .insert(&state.db)
    .await
    .unwrap();

    Ok((AxumStatusCode::OK, Json(data)))
}

#[utoipa::path(
    get,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [ListClusterElement]),
        (status = 404, body = [ErrorResponse]),
    ),
    security(
        ("api_key" = [])
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
) -> impl IntoResponse {
    let uuid = match uuid::Uuid::parse_str(&ctx.uuid) {
        Ok(uuid) => uuid,
        Err(_) => return Err((AxumStatusCode::INTERNAL_SERVER_ERROR, ())),
    };

    let res = match (user::Entity::find_by_id(uuid)
        .find_also_related(cluster::Entity)
        .all(&state.db))
    .await
    {
        Ok(v) => v,
        Err(_) => return Err((AxumStatusCode::BAD_REQUEST, ())),
    };

    let elems: Vec<ListClusterElement> = res
        .into_iter()
        .filter_map(|(_, cluster)| {
            cluster.map(|c| ListClusterElement {
                uuid: c.id.to_string(),
                name: c.name.clone(),
                encoded_id: Some(URL_SAFE.encode(c.id.as_bytes())),
            })
        })
        .collect();

    Ok((AxumStatusCode::OK, Json(elems)))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ClusterCreate {
    name: String,
    region: String,
    description: String,
}

#[derive(Serialize, ToSchema)]
pub struct ListClusterElement {
    uuid: String,
    name: String,
    encoded_id: Option<String>,
}
