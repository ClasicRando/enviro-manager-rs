use actix_web::{web, Scope};
use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};

use super::data::WorkflowRunTask;
use crate::{
    workflow::data::WorkflowId,
    workflow_run::{
        data::{TaskQueueRequest, WorkflowRun, WorkflowRunId},
        service::{TaskQueueService, WorkflowRunsService},
    },
};

pub fn workflow_runs_service<R>() -> Scope
where
    R: WorkflowRunsService + Send + Sync + 'static,
{
    web::scope("/workflow-runs")
        .route("/{workflow_run_id}", web::get().to(workflow_run::<R>))
        .route(
            "/tasks/{workflow_run_id}",
            web::get().to(workflow_run_tasks::<R>),
        )
        .route("", web::get().to(workflow_runs::<R>))
        .route(
            "/init/{workflow_id}",
            web::post().to(init_workflow_run::<R>),
        )
        .route(
            "/cancel/{workflow_run_id}",
            web::post().to(cancel_workflow_run::<R>),
        )
        .route(
            "/schedule/{workflow_run_id}",
            web::post().to(schedule_workflow_run::<R>),
        )
        .route(
            "/restart/{workflow_run_id}",
            web::post().to(restart_workflow_run::<R>),
        )
}

pub fn task_queue_service<Q, R>() -> Scope
where
    Q: TaskQueueService<WorkflowRunService = R> + Send + Sync + 'static,
    R: WorkflowRunsService + Send + Sync + 'static,
{
    web::scope("/task-queue")
        .route("/retry", web::post().to(task_queue_retry::<Q>))
        .route("/complete", web::post().to(task_queue_complete::<Q>))
}

/// API endpoint to fetch the specified workflow run by the `workflow_run_id`. Returns a single
/// [WorkflowRun] if the run can be found
async fn workflow_run<R>(
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

/// API endpoint to fetch the specified workflow run by the `workflow_run_id`. Returns a single
/// [WorkflowRun] if the run can be found
async fn workflow_run_tasks<R>(
    workflow_run_id: actix_web::web::Path<WorkflowRunId>,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<WorkflowRunTask>>
where
    R: WorkflowRunsService,
{
    let format = query.into_inner();
    match service.read_one(&workflow_run_id).await {
        Ok(workflow_run) => ApiResponse::success(workflow_run.tasks, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to fetch all workflow runs Returns a single
/// [WorkflowRun] if the run can be found
async fn workflow_runs<R>(
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<WorkflowRun>>
where
    R: WorkflowRunsService,
{
    let format = query.into_inner();
    match service.read_active().await {
        Ok(workflow_run) => ApiResponse::success(workflow_run, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to initialize a workflow run for the specified `workflow_id`. Returns the new
/// [WorkflowRun] if the `workflow_id` is valid and the init does not fail.
async fn init_workflow_run<R>(
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
async fn cancel_workflow_run<R>(
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
async fn schedule_workflow_run<R>(
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
async fn restart_workflow_run<R>(
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

/// API endpoint to retry the task queue entry specified by `request`
async fn task_queue_retry<T>(
    api_request: ApiRequest<TaskQueueRequest>,
    service: actix_web::web::Data<T>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<()>
where
    T: TaskQueueService,
{
    let format = query.into_inner();
    let request = api_request.into_inner();
    match service.retry_task(&request).await {
        Ok(_) => ApiResponse::message(
            String::from("Successfully set task queue record to retry. Workflow scheduled for run"),
            format.f,
        ),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint complete the task queue entry specified by `request`
async fn task_queue_complete<T>(
    api_request: ApiRequest<TaskQueueRequest>,
    service: actix_web::web::Data<T>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<()>
where
    T: TaskQueueService,
{
    let format = query.into_inner();
    let request = api_request.into_inner();
    match service.complete_task(&request).await {
        Ok(_) => ApiResponse::message(
            String::from(
                "Successfully set task queue record to complete. Workflow scheduled for run",
            ),
            format.f,
        ),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
