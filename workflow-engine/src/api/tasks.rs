use rocket::{
    get, post,
    serde::{json::Json, msgpack::MsgPack},
    State,
};

use super::utilities::{ApiFormatType, ApiResponse};
use crate::services::tasks::{Task, TaskId, TaskRequest, TasksService};

/// API endpoint to fetch all tasks. Return an array of [Task] entries
#[get("/tasks?<f>")]
pub async fn tasks(f: ApiFormatType, service: &State<TasksService>) -> ApiResponse<Vec<Task>> {
    match service.read_many().await {
        Ok(tasks) => ApiResponse::success(tasks, f),
        Err(error) => ApiResponse::error(error, f),
    }
}

/// API endpoint to fetch a task specified by `task_id`. Returns a single [Task] if a task with
/// that id exists
#[get("/tasks/<task_id>?<f>")]
pub async fn task(
    task_id: TaskId,
    f: ApiFormatType,
    service: &State<TasksService>,
) -> ApiResponse<Task> {
    match service.read_one(&task_id).await {
        Ok(task_option) => match task_option {
            Some(task) => ApiResponse::success(task, f),
            None => ApiResponse::failure(
                format!("Could not find record for task_id = {}", task_id),
                f,
            ),
        },
        Err(error) => ApiResponse::error(error, f),
    }
}

/// Create a task specified  by `request` regardless of the serialized format
async fn create_task(
    request: TaskRequest,
    service: &TasksService,
    format: ApiFormatType,
) -> ApiResponse<Task> {
    match service.create(&request).await {
        Ok(task) => ApiResponse::success(task, format),
        Err(error) => ApiResponse::error(error, format),
    }
}

/// API endpoint to create a new task specified by the MessagePack encoded `task`
#[post("/tasks?<f>", format = "msgpack", data = "<task>")]
pub async fn create_task_msgpack(
    task: MsgPack<TaskRequest>,
    f: ApiFormatType,
    service: &State<TasksService>,
) -> ApiResponse<Task> {
    create_task(task.0, service, f).await
}

/// API endpoint to create a new task specified by the JSON encoded `task`
#[post("/tasks?<f>", format = "json", data = "<task>")]
pub async fn create_task_json(
    task: Json<TaskRequest>,
    f: ApiFormatType,
    service: &State<TasksService>,
) -> ApiResponse<Task> {
    create_task(task.0, service, f).await
}
