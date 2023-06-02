use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};

use crate::services::tasks::{Task, TaskId, TaskRequest, TaskService};

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
