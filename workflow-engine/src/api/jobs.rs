use log::error;
use rocket::{
    get, post,
    serde::{json::Json, msgpack::MsgPack},
    State,
};

use super::utilities::{ApiResponse, FormatType};

use crate::services::jobs::{Job, JobId, JobRequest, JobsService};

#[get("/cron-jobs?<f>")]
pub async fn jobs(
    f: ApiResponse<FormatType>,
    service: &State<JobsService>,
) -> ApiResponse<Vec<Job>> {
    match service.read_many().await {
        Ok(jobs) => ApiResponse::success(jobs, f),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error, f)
        }
    }
}

#[get("/cron-jobs/<job_id>?<f>")]
pub async fn job(
    job_id: JobId,
    f: ApiResponse<FormatType>,
    service: &State<JobsService>,
) -> ApiResponse<Job> {
    match service.read_one(&job_id).await {
        Ok(Some(job)) => ApiResponse::success(job, f),
        Ok(None) => {
            ApiResponse::failure(format!("Could not find record for job_id = {}", job_id), f)
        }
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error, f)
        }
    }
}

async fn create_job(
    job: JobRequest,
    service: &State<JobsService>,
    f: ApiResponse<FormatType>,
) -> ApiResponse<Job> {
    match service.create(job).await {
        Ok(job) => ApiResponse::success(job, f),
        Err(error) => {
            error!("{}", error);
            ApiResponse::error(error, f)
        }
    }
}

#[post("/cron-jobs?<f>", format = "json", data = "<job>")]
pub async fn create_job_json(
    job: Json<JobRequest>,
    f: ApiResponse<FormatType>,
    service: &State<JobsService>,
) -> ApiResponse<Job> {
    create_job(job.0, service, f).await
}

#[post("/cron-jobs?<f>", format = "msgpack", data = "<job>")]
pub async fn create_job_msgpack(
    job: MsgPack<JobRequest>,
    f: ApiResponse<FormatType>,
    service: &State<JobsService>,
) -> ApiResponse<Job> {
    create_job(job.0, service, f).await
}
