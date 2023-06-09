use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};

use super::data::WorkflowUpdateRequest;
use crate::workflow::{
    data::{
        Task, TaskId, TaskRequest, Workflow, WorkflowCreateRequest, WorkflowDeprecationRequest,
        WorkflowId,
    },
    service::{TaskService, WorkflowsService},
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
        Ok(workflow) => ApiResponse::success(workflow, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to create a new workflow using encoded data from `workflow`
pub async fn create_workflow<W>(
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
pub async fn update_workflow<W>(
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

/// API endpoint to fetch all tasks. Return an array of [Task] entries
pub async fn tasks<T>(
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
pub async fn task<T>(
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
pub async fn create_task<T>(
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
