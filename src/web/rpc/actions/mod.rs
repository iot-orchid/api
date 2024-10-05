use super::error::Result;
use super::Ctx;
use super::ModelManager;
use axum_jrpc::error::JsonRpcError;
use axum_jrpc::JrpcResult;
use axum_jrpc::JsonRpcResponse;
#[allow(unused_imports)]
use entity;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use strum::EnumString;

// Enum representing the different actions available for microdevices
#[derive(Debug, EnumString, utoipa::ToSchema, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum MicrodeviceActions {
    Start,
    Stop,
    Restart,
    Reset,
    PowerOn,
    PowerOff,
}

// Implementation of actions that can be performed on microdevices
impl MicrodeviceActions {
    pub async fn execute(
        &self,
        _model_manager: &ModelManager,
        _ctx: &Ctx,
        _cluster_id: &String,
        id: axum_jrpc::Id,
        params: Value,
    ) -> JrpcResult {
        let parsed_params: MicroDeviceActionParams = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(e) => {
                return Err(JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        axum_jrpc::error::JsonRpcErrorReason::InvalidParams,
                        e.to_string(),
                        Value::Null,
                    ),
                ));
            }
        };

        // Different action responses for each method
        let response = match self {
            MicrodeviceActions::Start => Ok(JsonRpcResponse::success(
                id,
                serde_json::to_value(parsed_params).unwrap(),
            )),
            MicrodeviceActions::Stop => unimplemented(id),
            MicrodeviceActions::Restart => unimplemented(id),
            MicrodeviceActions::Reset => unimplemented(id),
            MicrodeviceActions::PowerOn => unimplemented(id),
            MicrodeviceActions::PowerOff => unimplemented(id),
        };

        response
    }
}

fn unimplemented(id: axum_jrpc::Id) -> JrpcResult {
    Err(JsonRpcResponse::error(
        id,
        JsonRpcError::new(
            axum_jrpc::error::JsonRpcErrorReason::MethodNotFound,
            "Method not implemented".to_string(),
            Value::Null,
        ),
    ))
}

/// Checks if device_id(s) exist in the cluster
#[allow(unused_variables)]
async fn check_device_id(
    model_manager: &ModelManager,
    cluster_id: &String,
    device_id: &MicrodeviceId,
) -> Result<()> {
    todo!()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum MicrodeviceId {
    Multiple(Vec<String>),
    Single(String),
}

#[derive(Debug, Deserialize, Serialize)]

struct MicroDeviceActionParams {
    cluster_wide: bool,
    device_id: Option<MicrodeviceId>,
}

// async fn start_device(mm: &ModelManager, ctx: &Ctx, cluster_id: &String, device_id: &String) {}
