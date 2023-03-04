use rocket::{
    get, patch, post,
    serde::{json::Json, msgpack::MsgPack},
    State,
};

use super::utilities::{ApiFormatType, ApiResponse};

use crate::services::workflows::{
    Workflow, WorkflowDeprecationRequest, WorkflowId, WorkflowRequest, WorkflowsService,
};

/// API endpoint to fetch all workflows. Returns an array of [WorkFlow] records.
#[get("/workflows?<f>")]
pub async fn workflows(
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<Vec<Workflow>> {
    match service.read_many().await {
        Ok(workflows) => ApiResponse::success(workflows, f),
        Err(error) => ApiResponse::error(error, f),
    }
}

/// API endpoint to fetch a workflow specified by `workflow_id`. Returns a single [Workflow] record
/// if any exists.
#[get("/workflows/<workflow_id>?<f>")]
pub async fn workflow(
    workflow_id: WorkflowId,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<Workflow> {
    match service.read_one(&workflow_id).await {
        Ok(Some(workflow)) => ApiResponse::success(workflow, f),
        Ok(None) => ApiResponse::failure(
            format!("Could not find record for workflow_id = {}", workflow_id),
            f,
        ),
        Err(error) => ApiResponse::error(error, f),
    }
}

/// Create a workflow specified  by `request` regardless of the serialized format
async fn create_workflow(
    request: WorkflowRequest,
    service: &WorkflowsService,
    format: ApiFormatType,
) -> ApiResponse<Workflow> {
    match service.create(request).await {
        Ok(workflow) => ApiResponse::success(workflow, format),
        Err(error) => ApiResponse::error(error, format),
    }
}

/// API endpoint to create a new workflow using MessagePack encoded data from `workflow`
#[post("/workflows?<f>", format = "msgpack", data = "<workflow>")]
pub async fn create_workflow_msgpack(
    workflow: MsgPack<WorkflowRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<Workflow> {
    create_workflow(workflow.0, service, f).await
}

/// API endpoint to create a new workflow using JSON encoded data from `workflow`
#[post("/workflows?<f>", format = "json", data = "<workflow>")]
pub async fn create_workflow_json(
    workflow: Json<WorkflowRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<Workflow> {
    create_workflow(workflow.0, service, f).await
}

/// Deprecate a workflow specified  by `request` regardless of the serialized format
async fn deprecate_workflow(
    request: WorkflowDeprecationRequest,
    service: &WorkflowsService,
    format: ApiFormatType,
) -> ApiResponse<()> {
    match service.deprecate(request).await {
        Ok(workflow_id) => ApiResponse::message(
            format!("Successfully deprecated workflow_id = {}", workflow_id),
            format,
        ),
        Err(error) => ApiResponse::error(error, format),
    }
}

/// API endpoint to deprecate a workflow specified by the MessagePack encoded data from `request`
#[patch("/workflows/deprecate?<f>", format = "msgpack", data = "<request>")]
pub async fn deprecate_workflow_msgpack(
    request: MsgPack<WorkflowDeprecationRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<()> {
    deprecate_workflow(request.0, service, f).await
}

/// API endpoint to deprecate a workflow specified by the JSON encoded data from `request`
#[patch("/workflows/deprecate?<f>", format = "json", data = "<request>")]
pub async fn deprecate_workflow_json(
    request: Json<WorkflowDeprecationRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<()> {
    deprecate_workflow(request.0, service, f).await
}
