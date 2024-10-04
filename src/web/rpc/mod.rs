// Enable warnings for performance and style from Clippy lints
#[warn(clippy::perf)]
#[warn(clippy::style)]
mod error;

use error::{Error, Result};
use crate::context::Ctx;
use crate::model::ModelManager;
use axum::extract::{Extension, Json as ExtractJson, Path, State};
use axum::Json;
use axum_jrpc::{Id, JrpcResult, JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;
use strum::EnumString;

// Struct to define an example JSON-RPC request for the API documentation
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

// Defines possible states for processing a request
#[derive(Debug, Clone)]
enum RequestProcessingState {
    Error(JrpcResult),
    Parsed((Id, MicrodeviceActions, Value)),
}

// Helper method to unwrap RequestProcessingState::Error variant, returns the JrpcResult
impl RequestProcessingState {
    fn unwrap(self) -> JrpcResult {
        match self {
            RequestProcessingState::Parsed(_) => panic!("called `unwrap()` on a `Parsed` variant"),
            RequestProcessingState::Error(r) => r, 
        }
    }
}

// Route handler for JSON-RPC requests on microdevices
#[utoipa::path(
    post,
    path = "/clusters/{clusterId}/devices/actions",
    tag = "Microdevices",
    params(
        ("clusterId" = String, Path, description="Cluster ID of an existing cluster"),
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
    State(model_manager): State<ModelManager>,  // State containing model manager
    Path(cluster_id): Path<String>,                   // Path parameter for the cluster ID
    Extension(ctx): Extension<Ctx>,                      // Extracted context (e.g., user or session data)
    ExtractJson(payload): Json<Value>,                 // JSON payload from the request
) -> Json<Value> {

    // Check if the incoming JSON payload is an array (batch request) or not
    match payload.as_array() {
        Some(reqs) => process_batch_requests(reqs, model_manager, ctx, cluster_id).await,
        None => {
            // If not an array, process the single request
            let req = parse_request(&payload);
            match req {
                RequestProcessingState::Parsed(params) => {
                    let res = execute_helper(&model_manager, &ctx, &cluster_id, params).await;
                    match res {
                        Ok(v) => Json(serde_json::to_value(v).unwrap()),
                        Err(e) => Json(serde_json::to_value(e).unwrap()),
                    }
                }
                RequestProcessingState::Error(e) => Json(serde_json::to_value(e).unwrap()),
            }
        }
    }
}

// Function to process a batch of requests
async fn process_batch_requests(
    reqs: &[Value], 
    model_manager: ModelManager, 
    ctx: Ctx, 
    cluster_id: String
) -> Json<Value> {
    
    // Parse the incoming requests into RequestProcessingState
    let mut req_states: Vec<RequestProcessingState> = reqs.iter().map(|req| parse_request(req)).collect();

    let mut future_tasks: Vec<_> = Vec::new(); // Futures for valid requests

    // Separate parsed requests from errors, and create future tasks for valid ones
    req_states.retain_mut(|req| {
        match req {
            RequestProcessingState::Parsed(params) => {
                let f = execute_helper(&model_manager, &ctx, &cluster_id, params.clone());
                future_tasks.push(f);
                false // Parsed requests go into the future task list
            },
            RequestProcessingState::Error(_) => true, // Keep error states for immediate response
        }
    });
    
    let future_results = futures::future::join_all(future_tasks).await;

    // Combine the requests: first handle errors, then append results of future tasks
    let mut responses: Vec<JrpcResult> = req_states
        .into_iter()
        .filter(|req| matches!(req, RequestProcessingState::Error(_)))
        .map(|v| v.unwrap())
        .collect();

    let future_responses: Vec<JrpcResult> = future_results.into_iter().collect();
    responses.extend(future_responses);

    // Map all JrpcResults to JsonRpcResponses
    let res: Vec<JsonRpcResponse> = responses.into_iter().map(|v| match v {
        Ok(v) => v,
        Err(e) => e,
    }).collect();
    
    // Return the response in JSON format
    Json(serde_json::to_value(res).unwrap()) 
}

// Function to parse a JSON request and return its processing state
fn parse_request(req: &Value) -> RequestProcessingState {
    match serde_json::from_value::<JsonRpcRequest>(req.clone()) {
        Ok(r) => {
            let action = match r.method.to_lowercase().parse::<MicrodeviceActions>() {
                Ok(a) => a,
                Err(e) => return RequestProcessingState::Error(Err(JsonRpcResponse::error(
                    r.id,
                    Error::InvalidMethod(format!("method `{}` is not a valid action", r.method)).into(),
                ))),
            };
            RequestProcessingState::Parsed((r.id, action, r.params))
        },
        Err(e) => RequestProcessingState::Error(Err(JsonRpcResponse::error(
            axum_jrpc::Id::None(()),
            Error::SerdeJson(e).into(),
        ))),
    }
}

// Function to execute a microdevice action and return a JSON-RPC result
pub async fn execute_helper(
    model_manager: &ModelManager,
    ctx: &Ctx,
    cluster_id: &String,
    (id, action, params): (Id, MicrodeviceActions, Value),
) -> JrpcResult {
    match action.execute(model_manager, ctx, cluster_id, &params).await {
        Ok(v) => Ok(JsonRpcResponse::success(id, v)),
        Err(e) => Err(JsonRpcResponse::error(id, e.into())),
    }
}

// Implementation of actions that can be performed on microdevices
impl MicrodeviceActions {
    async fn execute(
        &self, 
        _model_manager: &ModelManager, 
        ctx: &Ctx, 
        cluster_id: &String, 
        _params: &Value
    ) -> Result<Value> {
        // Different action responses for each method
        let response = match self {
            MicrodeviceActions::Start       => json_response("start", ctx, cluster_id),
            MicrodeviceActions::Stop        => json_response("stop", ctx, cluster_id),
            MicrodeviceActions::Restart     => json_response("restart", ctx, cluster_id),
            MicrodeviceActions::Reset       => json_response("reset", ctx, cluster_id),
            MicrodeviceActions::PowerOn     => json_response("power_on", ctx, cluster_id),
            MicrodeviceActions::PowerOff    => json_response("power_off", ctx, cluster_id),
        };
        Ok(response)
    }
}

// Helper function to create a consistent JSON response
fn json_response(action: &str, ctx: &Ctx, cluster_id: &String) -> Value {
    serde_json::json!({
        "message": action,
        "user": ctx.uuid,
        "cluster": cluster_id,
    })
}
