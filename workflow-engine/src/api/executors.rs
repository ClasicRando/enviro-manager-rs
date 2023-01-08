use rocket::{get, patch, State};

use super::utilities::{ApiResponse, FormatType};

use crate::services::executors::{Executor, ExecutorId, ExecutorsService};

#[get("/executors?<f>")]
pub async fn active_executors(
    f: ApiResponse<FormatType>,
    service: &State<ExecutorsService>,
) -> ApiResponse<Vec<Executor>> {
    match service.read_active().await {
        Ok(executors) => ApiResponse::success(executors, f),
        Err(error) => ApiResponse::error(error, f),
    }
}

#[patch("/executors/shutdown/<executor_id>?<f>")]
pub async fn shutdown_executor(
    executor_id: ExecutorId,
    f: ApiResponse<FormatType>,
    service: &State<ExecutorsService>,
) -> ApiResponse<Executor> {
    match service.shutdown(&executor_id).await {
        Ok(Some(executor)) => ApiResponse::success(executor, f),
        Ok(None) => ApiResponse::failure(
            format!(
                "Error while trying to shutdown executor_id = {}",
                executor_id
            ),
            f,
        ),
        Err(error) => ApiResponse::error(error, f),
    }
}

#[patch("/executors/cancel/<executor_id>?<f>")]
pub async fn cancel_executor(
    executor_id: ExecutorId,
    f: ApiResponse<FormatType>,
    service: &State<ExecutorsService>,
) -> ApiResponse<Executor> {
    match service.cancel(&executor_id).await {
        Ok(Some(executor)) => ApiResponse::success(executor, f),
        Ok(None) => ApiResponse::failure(
            format!("Error while trying to cancel executor_id = {}", executor_id),
            f,
        ),
        Err(error) => ApiResponse::error(error, f),
    }
}
