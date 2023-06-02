use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};

use crate::services::task_queue::{TaskQueueRequest, TaskQueueService};

/// API endpoint to retry the task queue entry specified by `request`
pub async fn task_queue_retry<T>(
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
pub async fn task_queue_complete<T>(
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
