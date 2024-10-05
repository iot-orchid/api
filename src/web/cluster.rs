use super::error::Result;
use crate::model::cluster::{ClusterBaseModelController, ClusterCreate, ClusterQuery};
use crate::model::ModelManager;
use crate::{context::Ctx, model::cluster::ClusterRecord};
use axum::{
    extract::{Extension, Json as ExtractJson, Query, State},
    response::Json,
};

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
    Ok(Json(
        ClusterBaseModelController::create_cluster(&state, &ctx, data).await?,
    ))
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
    params(
        ("uuid" = Option<String>, Query, description="Cluster UUID"),
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
    Ok(Json(
        ClusterBaseModelController::get_cluster(&state, &ctx, &params).await?,
    ))
}
