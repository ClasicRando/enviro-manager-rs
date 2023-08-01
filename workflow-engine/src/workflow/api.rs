use actix_web::{web, Scope};
use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};

use super::data::WorkflowUpdateRequest;
use crate::workflow::{
    data::{
        Task, TaskId, TaskRequest, Workflow, WorkflowCreateRequest, WorkflowDeprecationRequest,
        WorkflowId,
    },
    service::{TaskService, WorkflowsService},
};

pub fn workflows_service<W>() -> Scope
where
    W: WorkflowsService + Send + Sync + 'static,
{
    web::scope("/workflows")
        .service(
            web::resource("")
                .route(web::get().to(workflows::<W>))
                .route(web::post().to(create_workflow::<W>))
                .route(web::patch().to(update_workflow::<W>)),
        )
        .route("/{workflow_id}", web::get().to(workflow::<W>))
        .route("/deprecate", web::post().to(deprecate_workflow::<W>))
}

pub fn tasks_service<T>() -> Scope
where
    T: TaskService + Send + Sync + 'static,
{
    web::scope("/tasks")
        .service(
            web::resource("")
                .route(web::get().to(tasks::<T>))
                .route(web::post().to(create_task::<T>)),
        )
        .route("/{task_id}", web::get().to(task::<T>))
}

/// API endpoint to fetch all workflows. Returns an array of [WorkFlow] records.
async fn workflows<W>(
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
async fn workflow<W>(
    workflow_id: actix_web::web::Path<WorkflowId>,
    service: actix_web::web::Data<W>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Workflow>
where
    W: WorkflowsService,
{
    let format = query.into_inner();
    match service.read_one(&workflow_id).await {
        Ok(workflow) => ApiResponse::success(workflow, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to create a new workflow using encoded data from `workflow`
async fn create_workflow<W>(
    api_request: ApiRequest<WorkflowCreateRequest>,
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

/// API endpoint to update an existing workflow using encoded data from `workflow`
async fn update_workflow<W>(
    api_request: ApiRequest<WorkflowUpdateRequest>,
    service: actix_web::web::Data<W>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Workflow>
where
    W: WorkflowsService,
{
    let format = query.into_inner();
    let request = api_request.into_inner();
    match service.update_workflow(&request).await {
        Ok(workflow) => ApiResponse::success(workflow, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to deprecate a workflow specified by the encoded data from `request`
async fn deprecate_workflow<W>(
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

/// API endpoint to fetch all tasks. Return an array of [Task] entries
async fn tasks<T>(
    service: actix_web::web::Data<T>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<Task>>
where
    T: TaskService,
{
    let format = query.into_inner();
    match service.read_many().await {
        Ok(tasks) => ApiResponse::success(tasks, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to fetch a task specified by `task_id`. Returns a single [Task] if a task with
/// that id exists
async fn task<T>(
    task_id: actix_web::web::Path<TaskId>,
    service: actix_web::web::Data<T>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Task>
where
    T: TaskService,
{
    let format = query.into_inner();
    match service.read_one(&task_id).await {
        Ok(task) => ApiResponse::success(task, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to create a new task
async fn create_task<T>(
    api_request: ApiRequest<TaskRequest>,
    service: actix_web::web::Data<T>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Task>
where
    T: TaskService,
{
    let format = query.into_inner();
    let request = api_request.into_inner();
    match service.create_task(&request).await {
        Ok(task) => ApiResponse::success(task, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
