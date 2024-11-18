#[allow(unused_imports)]
use super::error::{Error, Result};
use crate::model::microdevice::{
    MicrodeviceBaseModelController as MicrodeviceBMC, MicrodeviceDeleteParams,
    MicrodeviceGetParams, MicrodeviceRecord, MicrodeviceUpdateParams,
};
use crate::model::ModelManager;
use crate::{context::Ctx, model::microdevice::MicrodeviceCreate};
use axum::{
    extract::{Extension, Json as ExtractJson, Path, Query, State},
    response::Json,
};
use serde::Serialize;
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
        ("include_cluster_id" = Option<bool>, Query, description="Include Cluster ID", example=true),
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
    State(mm): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_uuid): Path<String>,
    Query(params): Query<MicrodeviceGetParams>,
) -> Result<Json<Value>> {
    Ok(axum::Json(
        (serde_json::to_value(
            MicrodeviceBMC::get_microdevice_from_cluster(
                &mm,
                &ctx,
                cluster_uuid,
                match params.id {
                    Some(ids) => Some(vec![ids]),
                    None => None,
                },
                match params.name {
                    Some(name) => Some(vec![name]),
                    None => None,
                },
                params.include_topics,
                params.include_description,
                params.include_cluster_id,
            )
            .await?,
        ))
        .unwrap(),
    ))
}

#[allow(unused_variables)]
#[utoipa::path(
    delete,
    path = "/clusters/{clusterId}/devices",
    tag = "Microdevices",
    params(
        ("name" = Option<String>, Query, description="Microdevice Name"),
        ("id" = Option<i32>, Query, description="Microdevice ID"),
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
    State(mm): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_uuid): Path<String>,
    Query(params): Query<MicrodeviceDeleteParams>,
) -> Result<()> {
    Ok(MicrodeviceBMC::delete_microdevice_from_cluster(&mm, &ctx, cluster_uuid, params).await?)
}

#[derive(Serialize, ToSchema)]
pub struct Topic {
    topic: String,
}

/// Add a topic to a microdevice
///
/// This endpoint allows you to add a topic to a microdevice. The topic will be used to filter messages that are sent to the microdevice.
///
#[allow(unused_variables)]
#[utoipa::path(
    put,
    path = "/clusters/{clusterId}/device/{microdeviceId}",
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
pub async fn update_device(
    State(mm): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    Path((cluster_id, microdevice_id)): Path<(String, i32)>,
    ExtractJson(data): Json<MicrodeviceUpdateParams>,
) -> Result<Json<MicrodeviceRecord>> {
    Ok(Json(
        MicrodeviceBMC::update_microdevice_in_cluster(&mm, &ctx, cluster_id, microdevice_id, data)
            .await?,
    ))
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
    State(mm): State<ModelManager>,
    Extension(ctx): Extension<Ctx>,
    Path(cluster_uuid): Path<String>,
    ExtractJson(data): Json<MicrodeviceCreate>,
) -> Result<Json<MicrodeviceRecord>> {
    Ok(Json(
        MicrodeviceBMC::create_microdevice(&mm, &ctx, cluster_uuid, data).await?,
    ))
}
