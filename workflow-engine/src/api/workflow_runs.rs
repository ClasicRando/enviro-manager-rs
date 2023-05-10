use common::api::ApiResponse;

use crate::services::{
    workflow_runs::{WorkflowRun, WorkflowRunId, WorkflowRunsService},
    workflows::WorkflowId,
};

/// API endpoint to fetch the specified workflow run by the `workflow_run_id`. Returns a single
/// [WorkflowRun] if the run can be found
pub async fn workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    match service.read_one(&workflow_run_id).await {
        Ok(Some(workflow_run)) => ApiResponse::success(workflow_run),
        Ok(None) => ApiResponse::failure(format!(
            "Could not find record for workflow_run_id = {}",
            workflow_run_id
        )),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to initialize a workflow run for the specified `workflow_id`. Returns the new
/// [WorkflowRun] if the `workflow_id` is valid and the init does not fail.
// #[post("/workflow_runs/init/<workflow_id>?<f>")]
pub async fn init_workflow_run<R>(
    workflow_id: actix_web::web::Path<WorkflowId>,
    service: actix_web::web::Data<R>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    match service.initialize(&workflow_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to cancel the workflow run specified by the `workflow_run_id`. Returns the
/// canceled [WorkflowRun] if the operation was a success.
// #[patch("/workflow_runs/cancel/<workflow_run_id>?<f>")]
pub async fn cancel_workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    match service.cancel(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to set a workflow run specified by `workflow_run_id` as `Scheduled`. Returns the
/// [WorkflowRun] if the operation was successful
// #[patch("/workflow_runs/schedule/<workflow_run_id>?<f>")]
pub async fn schedule_workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    match service.schedule(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to restart a workflow run specified by `workflow_run_id`. Returns the
/// [WorkflowRun] if the operation was successful
// #[put("/workflow_runs/restart/<workflow_run_id>?<f>")]
pub async fn restart_workflow_run<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
) -> ApiResponse<WorkflowRun>
where
    R: WorkflowRunsService,
{
    match service.restart(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run),
        Err(error) => ApiResponse::error(error),
    }
}
