use common::api::ApiResponse;
use log::error;

use crate::services::jobs::{Job, JobId, JobRequest, JobService};

/// API endpoint to fetch all `Job`s currently registered
pub async fn jobs<J>(service: actix_web::web::Data<J>) -> ApiResponse<Vec<Job>>
where
    J: JobService,
{
    match service.read_many().await {
        Ok(jobs) => ApiResponse::success(jobs),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error)
        }
    }
}

/// API endpoint to fetch the [Job] details of a cron job specified by `job_id`
pub async fn job<J>(
    job_id: actix_web::web::Path<JobId>,
    service: actix_web::web::Data<J>,
) -> ApiResponse<Job>
where
    J: JobService,
{
    match service.read_one(&job_id).await {
        Ok(job) => ApiResponse::success(job),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error)
        }
    }
}

/// API endpoint to create a new [Job] using the provided [JobRequest] details
pub async fn create_job<J>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<J>,
) -> ApiResponse<Job>
where
    J: JobService,
{
    let job: JobRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize job request. Error: {}",
                error
            ));
        }
    };
    match service.create_job(&job).await {
        Ok(job) => ApiResponse::success(job),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error)
        }
    }
}
