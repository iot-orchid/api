use super::error::Result;
use crate::model::cluster::{
    ClusterBaseModelController, ClusterCreate, ClusterDelete, ClusterQuery,
};
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

/// Get a all clusters or a cluster by UUID
///
/// If `uuid` is provided [as a query parameter], it will return a single cluster by UUID.
/// Otherwise, it will return all clusters belonging to the currently authenticated user.
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

/// Delete a cluster(s) by UUID
///
/// Accepts a list of UUIDs to delete in a JSON payload.
///
/// To delete a single cluster, provide a single UUID using the `id` field.
///
/// To delete multiple clusters, provide a list of UUIDs using the `ids` field.
#[utoipa::path(
    delete,
    path = "/clusters",
    tag = "Clusters",
    responses(
        (status = 200, body = [ClusterRecord]),
        (status = 401),
        (status = 400),
    ),
)]
pub async fn delete(
    State(state): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    ExtractJson(data): Json<ClusterDelete>,
) -> Result<Json<serde_json::Value>> {
    let n_deleted = ClusterBaseModelController::delete_cluster(&state, &ctx, data).await?;

    Ok(Json(serde_json::json!(
        {
            "deleted": n_deleted,
            "message": "Cluster(s) deleted successfully",
        }
    )))
}
