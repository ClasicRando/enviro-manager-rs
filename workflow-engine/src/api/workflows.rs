use rocket::{
    get, patch, post,
    serde::{json::Json, msgpack::MsgPack},
    State,
};

use super::utilities::{ApiResponse, ApiFormatType};

use crate::services::workflows::{
    Workflow, WorkflowDeprecationRequest, WorkflowId, WorkflowRequest, WorkflowsService,
};

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

#[post("/workflows?<f>", format = "msgpack", data = "<workflow>")]
pub async fn create_workflow_msgpack(
    workflow: MsgPack<WorkflowRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<Workflow> {
    create_workflow(workflow.0, service, f).await
}

#[post("/workflows?<f>", format = "json", data = "<workflow>")]
pub async fn create_workflow_json(
    workflow: Json<WorkflowRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<Workflow> {
    create_workflow(workflow.0, service, f).await
}

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

#[patch("/workflows/deprecate?<f>", format = "msgpack", data = "<request>")]
pub async fn deprecate_workflow_msgpack(
    request: MsgPack<WorkflowDeprecationRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<()> {
    deprecate_workflow(request.0, service, f).await
}

#[patch("/workflows/deprecate?<f>", format = "json", data = "<request>")]
pub async fn deprecate_workflow_json(
    request: Json<WorkflowDeprecationRequest>,
    f: ApiFormatType,
    service: &State<WorkflowsService>,
) -> ApiResponse<()> {
    deprecate_workflow(request.0, service, f).await
}
