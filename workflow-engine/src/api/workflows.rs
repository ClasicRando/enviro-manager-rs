use common::api::ApiResponse;
use log::error;

use crate::services::workflows::{
    Workflow, WorkflowDeprecationRequest, WorkflowId, WorkflowRequest, WorkflowsService,
};

/// API endpoint to fetch all workflows. Returns an array of [WorkFlow] records.
pub async fn workflows<W>(service: actix_web::web::Data<W>) -> ApiResponse<Vec<Workflow>>
where
    W: WorkflowsService,
{
    match service.read_many().await {
        Ok(workflows) => ApiResponse::success(workflows),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to fetch a workflow specified by `workflow_id`. Returns a single [Workflow] record
/// if any exists.
pub async fn workflow<W>(
    workflow_id: actix_web::web::Path<WorkflowId>,
    service: actix_web::web::Data<W>,
) -> ApiResponse<Workflow>
where
    W: WorkflowsService,
{
    match service.read_one(&workflow_id).await {
        Ok(Some(workflow)) => ApiResponse::success(workflow),
        Ok(None) => ApiResponse::failure(format!(
            "Could not find record for workflow_id = {}",
            workflow_id
        )),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to create a new workflow using encoded data from `workflow`
pub async fn create_workflow<W>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<W>,
) -> ApiResponse<Workflow>
where
    W: WorkflowsService,
{
    let request: WorkflowRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could not deserialize workflow request. Error: {}",
                error
            ));
        }
    };
    match service.create(request).await {
        Ok(workflow) => ApiResponse::success(workflow),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to deprecate a workflow specified by the encoded data from `request`
pub async fn deprecate_workflow<W>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<W>,
) -> ApiResponse<()>
where
    W: WorkflowsService,
{
    let request: WorkflowDeprecationRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could not deserialize workflow deprecation request. Error: {}",
                error
            ));
        }
    };
    match service.deprecate(request).await {
        Ok(workflow_id) => ApiResponse::message(format!(
            "Successfully deprecated workflow_id = {}",
            workflow_id
        )),
        Err(error) => ApiResponse::error(error),
    }
}
