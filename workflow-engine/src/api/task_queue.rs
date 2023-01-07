use rocket::{
    patch,
    serde::{json::Json, msgpack::MsgPack},
    State,
};

use super::utilities::{ApiResponse, FormatType};

use crate::services::task_queue::{TaskQueueRequest, TaskQueueService};

async fn task_queue_retry(
    request: TaskQueueRequest,
    service: &TaskQueueService,
    format: ApiResponse<FormatType>,
) -> ApiResponse<()> {
    match service.retry_task(request).await {
        Ok(_) => ApiResponse::message(
            String::from("Successfully set task queue record to retry. Workflow scheduled for run"),
            format,
        ),
        Err(error) => ApiResponse::error(error, format),
    }
}

#[patch("/task-queue/retry?<f>", format = "json", data = "<request>")]
pub async fn task_queue_retry_json(
    request: Json<TaskQueueRequest>,
    f: ApiResponse<FormatType>,
    service: &State<TaskQueueService>,
) -> ApiResponse<()> {
    task_queue_retry(request.0, service, f).await
}

#[patch("/task-queue/retry?<f>", format = "msgpack", data = "<request>")]
pub async fn task_queue_retry_msgpack(
    request: MsgPack<TaskQueueRequest>,
    f: ApiResponse<FormatType>,
    service: &State<TaskQueueService>,
) -> ApiResponse<()> {
    task_queue_retry(request.0, service, f).await
}

async fn task_queue_complete(
    request: TaskQueueRequest,
    service: &TaskQueueService,
    format: ApiResponse<FormatType>,
) -> ApiResponse<()> {
    match service.complete_task(request).await {
        Ok(_) => ApiResponse::message(
            String::from(
                "Successfully set task queue record to complete. Workflow scheduled for run",
            ),
            format,
        ),
        Err(error) => ApiResponse::error(error, format),
    }
}

#[patch("/task-queue/complete?<f>", format = "json", data = "<request>")]
pub async fn task_queue_complete_json(
    request: Json<TaskQueueRequest>,
    f: ApiResponse<FormatType>,
    service: &State<TaskQueueService>,
) -> ApiResponse<()> {
    task_queue_complete(request.0, service, f).await
}

#[patch("/task-queue/complete?<f>", format = "msgpack", data = "<request>")]
pub async fn task_queue_complete_msgpack(
    request: MsgPack<TaskQueueRequest>,
    f: ApiResponse<FormatType>,
    service: &State<TaskQueueService>,
) -> ApiResponse<()> {
    task_queue_complete(request.0, service, f).await
}
