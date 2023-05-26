use common::api::ApiResponse;
use log::error;

use crate::services::task_queue::{TaskQueueRequest, TaskQueueService};

/// API endpoint to retry the task queue entry specified by `request`
pub async fn task_queue_retry<T>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<T>,
) -> ApiResponse<()>
where
    T: TaskQueueService,
{
    let request: TaskQueueRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could not deserialize job request. Error: {}",
                error
            ));
        }
    };
    match service.retry_task(request).await {
        Ok(_) => ApiResponse::message(String::from(
            "Successfully set task queue record to retry. Workflow scheduled for run",
        )),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint complete the task queue entry specified by `request`
pub async fn task_queue_complete<T>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<T>,
) -> ApiResponse<()>
where
    T: TaskQueueService,
{
    let request: TaskQueueRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could not deserialize job request. Error: {}",
                error
            ));
        }
    };
    match service.complete_task(request).await {
        Ok(_) => ApiResponse::message(String::from(
            "Successfully set task queue record to complete. Workflow scheduled for run",
        )),
        Err(error) => ApiResponse::error(error),
    }
}
