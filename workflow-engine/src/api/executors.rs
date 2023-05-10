use common::api::ApiResponse;

use crate::services::executors::{Executor, ExecutorId, ExecutorsService};

/// API endpoint to fetch all active executors
pub async fn active_executors<E>(service: actix_web::web::Data<E>) -> ApiResponse<Vec<Executor>>
where
    E: ExecutorsService,
{
    match service.read_active().await {
        Ok(executors) => ApiResponse::success(executors),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to start the graceful shutdown of the executor specified by `executor_id`
pub async fn shutdown_executor<E>(
    executor_id: actix_web::web::Path<ExecutorId>,
    service: actix_web::web::Data<E>,
) -> ApiResponse<Executor>
where
    E: ExecutorsService,
{
    match service.shutdown(&executor_id).await {
        Ok(Some(executor)) => ApiResponse::success(executor),
        Ok(None) => ApiResponse::failure(format!(
            "Error while trying to shutdown executor_id = {}",
            executor_id
        )),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to the forceful shutdown of the executor specified by `executor_id`
pub async fn cancel_executor<E>(
    executor_id: actix_web::web::Path<ExecutorId>,
    service: actix_web::web::Data<E>,
) -> ApiResponse<Executor>
where
    E: ExecutorsService,
{
    match service.cancel(&executor_id).await {
        Ok(Some(executor)) => ApiResponse::success(executor),
        Ok(None) => ApiResponse::failure(format!(
            "Error while trying to cancel executor_id = {}",
            executor_id
        )),
        Err(error) => ApiResponse::error(error),
    }
}
