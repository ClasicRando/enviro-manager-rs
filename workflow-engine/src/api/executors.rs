use common::api::{ApiResponse, QueryApiFormat};

use crate::services::executors::{Executor, ExecutorId, ExecutorService};

/// API endpoint to fetch all active executors
pub async fn active_executors<E>(
    service: actix_web::web::Data<E>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<Executor>>
where
    E: ExecutorService,
{
    let format = query.into_inner();
    match service.read_active().await {
        Ok(executors) => ApiResponse::success(executors, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to start the graceful shutdown of the executor specified by `executor_id`
pub async fn shutdown_executor<E>(
    executor_id: actix_web::web::Path<ExecutorId>,
    service: actix_web::web::Data<E>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Executor>
where
    E: ExecutorService,
{
    let format = query.into_inner();
    match service.shutdown(&executor_id).await {
        Ok(executor) => ApiResponse::success(executor, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to the forceful shutdown of the executor specified by `executor_id`
pub async fn cancel_executor<E>(
    executor_id: actix_web::web::Path<ExecutorId>,
    service: actix_web::web::Data<E>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Executor>
where
    E: ExecutorService,
{
    let format = query.into_inner();
    match service.cancel(&executor_id).await {
        Ok(executor) => ApiResponse::success(executor, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
