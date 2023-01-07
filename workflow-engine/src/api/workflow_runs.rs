use rocket::{get, patch, post, put, State};

use super::utilities::{ApiResponse, FormatType};

use crate::services::workflow_runs::{WorkflowRun, WorkflowRunsService};

#[get("/workflow_runs/<workflow_run_id>?<f>")]
pub async fn workflow_runs(
    workflow_run_id: i64,
    f: ApiResponse<FormatType>,
    service: &State<WorkflowRunsService>,
) -> ApiResponse<WorkflowRun> {
    match service.read_one(workflow_run_id).await {
        Ok(Some(workflow_run)) => ApiResponse::success(workflow_run, f),
        Ok(None) => ApiResponse::failure(
            format!(
                "Could not find record for workflow_run_id = {}",
                workflow_run_id
            ),
            f,
        ),
        Err(error) => ApiResponse::error(error, f),
    }
}

#[post("/workflow_runs/init/<workflow_id>?<f>")]
pub async fn init_workflow_run(
    workflow_id: i64,
    f: ApiResponse<FormatType>,
    service: &State<WorkflowRunsService>,
) -> ApiResponse<WorkflowRun> {
    match service.initialize(workflow_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run, f),
        Err(error) => ApiResponse::error(error, f),
    }
}

#[patch("/workflow_runs/cancel/<workflow_run_id>?<f>")]
pub async fn cancel_workflow_run(
    workflow_run_id: i64,
    f: ApiResponse<FormatType>,
    service: &State<WorkflowRunsService>,
) -> ApiResponse<WorkflowRun> {
    match service.cancel(workflow_run_id).await {
        Ok(Some(workflow_run)) => ApiResponse::success(workflow_run, f),
        Ok(None) => ApiResponse::failure(
            format!(
                "Error while trying to cancel a workflow run for workflow_run_id = {}",
                workflow_run_id
            ),
            f,
        ),
        Err(error) => ApiResponse::error(error, f),
    }
}

#[patch("/workflow_runs/schedule/<workflow_run_id>?<f>")]
pub async fn schedule_workflow_run(
    workflow_run_id: i64,
    f: ApiResponse<FormatType>,
    service: &State<WorkflowRunsService>,
) -> ApiResponse<WorkflowRun> {
    match service.schedule(workflow_run_id).await {
        Ok(Some(workflow_run)) => ApiResponse::success(workflow_run, f),
        Ok(None) => ApiResponse::failure(
            format!(
                "Error while trying to cancel workflow_run_id = {}",
                workflow_run_id
            ),
            f,
        ),
        Err(error) => ApiResponse::error(error, f),
    }
}

#[put("/workflow_runs/restart/<workflow_run_id>?<f>")]
pub async fn restart_workflow_run(
    workflow_run_id: i64,
    f: ApiResponse<FormatType>,
    service: &State<WorkflowRunsService>,
) -> ApiResponse<WorkflowRun> {
    match service.restart(workflow_run_id).await {
        Ok(Some(workflow_run)) => ApiResponse::success(workflow_run, f),
        Ok(None) => ApiResponse::failure(
            format!(
                "Error while trying to restart workflow_run_id = {}",
                workflow_run_id
            ),
            f,
        ),
        Err(error) => ApiResponse::error(error, f),
    }
}
