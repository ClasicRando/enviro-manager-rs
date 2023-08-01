use actix_web::{web, Scope};
use common::api::{ApiResponse, QueryApiFormat};

use crate::executor::{
    data::{Executor, ExecutorId},
    service::ExecutorService,
};

pub fn service<E>() -> Scope
where
    E: ExecutorService + Send + Sync + 'static,
{
    web::scope("/executors")
        .route("", web::get().to(active_executors::<E>))
        .route(
            "/shutdown/{executor_id}",
            web::post().to(shutdown_executor::<E>),
        )
        .route(
            "/cancel/{executor_id}",
            web::post().to(cancel_executor::<E>),
        )
        .route("/clean", web::post().to(clean_executors::<E>))
}

/// API endpoint to fetch all active executors
async fn active_executors<E>(
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
async fn shutdown_executor<E>(
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
async fn cancel_executor<E>(
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

/// API endpoint to perform a cleaning of all inactive but not closed executors
async fn clean_executors<E>(
    service: actix_web::web::Data<E>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Executor>
where
    E: ExecutorService,
{
    let format = query.into_inner();
    match service.clean_executors().await {
        Ok(_) => ApiResponse::message("Successfully cleaned executors".to_owned(), format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
