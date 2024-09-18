#[warn(clippy::perf)]
#[warn(clippy::style)]
use super::error::{Result, Error};
use crate::context::Ctx;
use crate::model::ModelManager;
use axum::extract::{Extension, Json as ExtractJson, Path, State};
use axum::Json;
use axum_jrpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;
use strum::EnumString;

#[utoipa::path(
    post,
    path = "/clusters/{clusterId}/devices/actions",
    tag = "Microdevices",
    params(
        ("clusterId" = String, Path, description="Cluster ID a existing cluster"),
    ),
    request_body(content = JrpcExample, description = "JSON-RPC Request",),
    responses(
        (status = 200),
        (status = 400, 
            body = String, 
            description = "Message describing what is missing in the JSON-RPC request that caused a malformed request",
        ),
    ),
)]
pub async fn rpc_handler(
    State(model_manager): State<ModelManager>,
    Path(cluster_id): Path<String>,
    Extension(ctx): Extension<Ctx>,
    ExtractJson(payload): Json<Value>,
) -> Result<Json<Value>> {
    match payload.as_array() {
        Some(batch) => {
            let batch_reqs: Vec<_> = batch.iter().map( |req| 
                // TODO: Parse the request here to validate its structure before execution.
                // This ensures that malformed requests are identified early, preventing the execution of valid requests
                // followed by a failure on the malformed one. This avoids losing responses from successfully executed requests
                // due to the failure in try_join_all.
                exectute_method(&model_manager, &ctx, &cluster_id, req)).collect();

            let rpc_responses = futures::future::try_join_all(batch_reqs).await?;

            Ok(Json(serde_json::to_value(rpc_responses)?))
        }
        // TODO: Parse the request here to validate its structure before execution.
        None => Ok(Json(exectute_method(&model_manager, &ctx, &cluster_id, &payload).await?)),
    }
}

pub async fn exectute_method(model_manager: &ModelManager, ctx: &Ctx, cluster_id: &String, req: &Value) -> Result<Value> {
    let req: JsonRpcRequest = serde_json::from_value(req.clone())?;
    let action = req.method.parse::<MicrodeviceActions>().map_err(|_| Error::InvalidMethod(req.method.clone()))?;

    let action_results = action.execute(model_manager, ctx, cluster_id, req.params).await?;
    Ok(serde_json::to_value(JsonRpcResponse::success(
        req.id, action_results,
    ))?)
}

#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub struct JrpcExample {
    #[schema(example = "2.0")]
    jsonrpc: String,
    #[schema(example = "<id>")]
    id: String,
    #[schema(example = "<method>")]
    method: MicrodeviceActions,
    params: Vec<String>,
}

#[derive(Debug, EnumString, utoipa::ToSchema)]
#[strum(serialize_all = "snake_case")]
pub enum MicrodeviceActions {
    Start,    
    Stop,
    Restart,
    Reset,
    PowerOn,
    PowerOff,
}


impl MicrodeviceActions {
    async fn execute(&self, _model_manager: &ModelManager, ctx: &Ctx, cluster_id: &String, _params: Value) -> Result<Value> {
        match self {
            MicrodeviceActions::Start => Ok(serde_json::json!({
                "message": "start",
                "user": ctx.uuid,
                "cluster": cluster_id,
            })),

            MicrodeviceActions::Stop => Ok(serde_json::json!({
                "message": "stop",
                "user": ctx.uuid,
                "cluster": cluster_id,
            })),

            MicrodeviceActions::Restart => Ok(serde_json::json!({
                "message": "restart",
                "user": ctx.uuid,
                "cluster": cluster_id,
            })),

            MicrodeviceActions::Reset => Ok(serde_json::json!({
                "message": "reset",
                "user": ctx.uuid,
                "cluster": cluster_id,
            })),

            MicrodeviceActions::PowerOn => Ok(serde_json::json!({
                "message": "power_on",
                "user": ctx.uuid,
                "cluster": cluster_id,
            })),

            MicrodeviceActions::PowerOff => Ok(serde_json::json!({
                "message": "power_off",
                "user": ctx.uuid,
                "cluster": cluster_id,
            })),
        }
    }
}