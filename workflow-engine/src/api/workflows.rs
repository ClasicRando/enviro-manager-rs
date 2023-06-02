use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};

use crate::services::workflows::{
    Workflow, WorkflowDeprecationRequest, WorkflowId, WorkflowRequest, WorkflowsService,
};

/// API endpoint to fetch all workflows. Returns an array of [WorkFlow] records.
pub async fn workflows<W>(
    service: actix_web::web::Data<W>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<Workflow>>
where
    W: WorkflowsService,
{
    let format = query.into_inner();
    match service.read_many().await {
        Ok(workflows) => ApiResponse::success(workflows, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to fetch a workflow specified by `workflow_id`. Returns a single [Workflow] record
/// if any exists.
pub async fn workflow<W>(
    workflow_id: actix_web::web::Path<WorkflowId>,
    service: actix_web::web::Data<W>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Workflow>
where
    W: WorkflowsService,
{
    let format = query.into_inner();
    match service.read_one(&workflow_id).await {
        Ok(Some(workflow)) => ApiResponse::success(workflow, format.f),
        Ok(None) => ApiResponse::failure(
            format!("Could not find record for workflow_id = {}", workflow_id),
            format.f,
        ),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to create a new workflow using encoded data from `workflow`
pub async fn create_workflow<W>(
    api_request: ApiRequest<WorkflowRequest>,
    service: actix_web::web::Data<W>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Workflow>
where
    W: WorkflowsService,
{
    let format = query.into_inner();
    let request = api_request.into_inner();
    match service.create_workflow(&request).await {
        Ok(workflow) => ApiResponse::success(workflow, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to deprecate a workflow specified by the encoded data from `request`
pub async fn deprecate_workflow<W>(
    api_request: ApiRequest<WorkflowDeprecationRequest>,
    service: actix_web::web::Data<W>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<()>
where
    W: WorkflowsService,
{
    let format = query.into_inner();
    let request = api_request.into_inner();
    match service.deprecate(&request).await {
        Ok(workflow_id) => ApiResponse::message(
            format!("Successfully deprecated workflow_id = {}", workflow_id),
            format.f,
        ),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
