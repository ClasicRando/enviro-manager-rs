use common::api::{ApiResponse, QueryApiFormat};

use crate::services::{
    workflow_runs::{WorkflowRun, WorkflowRunId, WorkflowRunsService},
    workflows::WorkflowId,
};

/// API endpoint to fetch the specified workflow run by the `workflow_run_id`. Returns a single
/// [WorkflowRun] if the run can be found
pub async fn workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    let format = query.into_inner();
    match service.read_one(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to initialize a workflow run for the specified `workflow_id`. Returns the new
/// [WorkflowRun] if the `workflow_id` is valid and the init does not fail.
pub async fn init_workflow_run<R>(
    workflow_id: actix_web::web::Path<WorkflowId>,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    let format = query.into_inner();
    match service.initialize(&workflow_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to cancel the workflow run specified by the `workflow_run_id`. Returns the
/// canceled [WorkflowRun] if the operation was a success.
pub async fn cancel_workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    let format = query.into_inner();
    match service.cancel(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to set a workflow run specified by `workflow_run_id` as `Scheduled`. Returns the
/// [WorkflowRun] if the operation was successful
pub async fn schedule_workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    let format = query.into_inner();
    match service.schedule(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to restart a workflow run specified by `workflow_run_id`. Returns the
/// [WorkflowRun] if the operation was successful
pub async fn restart_workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    let format = query.into_inner();
    match service.restart(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
