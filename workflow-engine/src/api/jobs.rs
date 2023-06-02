use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};
use log::error;

use crate::services::jobs::{Job, JobId, JobRequest, JobService};

/// API endpoint to fetch all `Job`s currently registered
pub async fn jobs<J>(
    service: actix_web::web::Data<J>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<Job>>
where
    J: JobService,
{
    let format = query.into_inner();
    match service.read_many().await {
        Ok(jobs) => ApiResponse::success(jobs, format.f),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error, format.f)
        }
    }
}

/// API endpoint to fetch the [Job] details of a cron job specified by `job_id`
pub async fn job<J>(
    job_id: actix_web::web::Path<JobId>,
    service: actix_web::web::Data<J>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Job>
where
    J: JobService,
{
    let format = query.into_inner();
    match service.read_one(&job_id).await {
        Ok(job) => ApiResponse::success(job, format.f),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error, format.f)
        }
    }
}

/// API endpoint to create a new [Job] using the provided [JobRequest] details
pub async fn create_job<J>(
    api_request: ApiRequest<JobRequest>,
    service: actix_web::web::Data<J>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Job>
where
    J: JobService,
{
    let format = query.into_inner();
    let job = api_request.into_inner();
    match service.create_job(&job).await {
        Ok(job) => ApiResponse::success(job, format.f),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error, format.f)
        }
    }
}
