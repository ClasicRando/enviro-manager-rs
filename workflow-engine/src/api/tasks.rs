use common::api::ApiResponse;
use log::error;

use crate::services::tasks::{Task, TaskId, TaskRequest, TaskService};

/// API endpoint to fetch all tasks. Return an array of [Task] entries
pub async fn tasks<T>(service: actix_web::web::Data<T>) -> ApiResponse<Vec<Task>>
where
    T: TaskService,
{
    match service.read_many().await {
        Ok(tasks) => ApiResponse::success(tasks),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to fetch a task specified by `task_id`. Returns a single [Task] if a task with
/// that id exists
pub async fn task<T>(
    task_id: actix_web::web::Path<TaskId>,
    service: actix_web::web::Data<T>,
) -> ApiResponse<Task>
where
    T: TaskService,
{
    match service.read_one(&task_id).await {
        Ok(task) => ApiResponse::success(task),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to create a new task
pub async fn create_task<T>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<T>,
) -> ApiResponse<Task>
where
    T: TaskService,
{
    let request: TaskRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could not deserialize task request. Error: {}",
                error
            ));
        }
    };
    match service.create_task(&request).await {
        Ok(task) => ApiResponse::success(task),
        Err(error) => ApiResponse::error(error),
    }
}
