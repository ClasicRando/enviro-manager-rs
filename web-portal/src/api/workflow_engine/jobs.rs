use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use serde::Deserialize;
use workflow_engine::{
    job::data::{Job, JobRequest, JobTypeEnum},
    workflow::data::WorkflowId,
};

use crate::{
    components::workflow_engine::main_page::{Jobs, JobsTab},
    extract_session_uid, utils, ServerFnError,
};

pub fn service() -> actix_web::Scope {
    web::scope("/jobs")
        .service(
            web::resource("")
                .route(web::get().to(jobs))
                .route(web::post().to(create_job)),
        )
        .route("/create-modal", web::post().to(create_job_modal))
        .route("/tab", web::get().to(jobs_tab))
}

async fn jobs_html(session: Session, is_tab: bool) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    let jobs = match get_jobs().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = if is_tab {
        leptos::ssr::render_to_string(|cx| {
            view! { cx, <JobsTab jobs=jobs/> }
        })
    } else {
        leptos::ssr::render_to_string(|cx| {
            view! { cx, <Jobs jobs=jobs/> }
        })
    };
    utils::html_chunk!(html)
}

async fn jobs(session: Session) -> HttpResponse {
    jobs_html(session, false).await
}

async fn jobs_tab(session: Session) -> HttpResponse {
    jobs_html(session, true).await
}

async fn get_jobs() -> Result<Vec<Job>, ServerFnError> {
    let jobs_response = utils::api_request(
        "http://127.0.0.1:8000/api/v1/jobs?f=msgpack",
        Method::GET,
        None::<String>,
        None::<()>,
    )
    .await?;
    let jobs = match jobs_response {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            return utils::server_fn_error!(message)
        }
    };
    Ok(jobs)
}

async fn create_job_modal(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    jobs_html(session, false).await
}

#[derive(Deserialize, Debug)]
struct CreateJob {
    workflow_id: WorkflowId,
    maintainer: String,
    job_type: JobTypeEnum,
    #[serde(default)]
    next_run: Option<String>,
}

async fn create_job(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    jobs_html(session, false).await
}

async fn post_create_job(job_request: JobRequest) -> Result<(), ServerFnError> {
    let clean_executors_response: ApiResponseBody<Job> = utils::api_request(
        "http://127.0.0.1:8000/api/v1/jobs?f=msgpack",
        Method::POST,
        None::<String>,
        Some(job_request),
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(job) => {
            log::info!("Create new job: {}", job.job_id);
            Ok(())
        }
        ApiResponseBody::Message(message) => {
            log::info!("{message}");
            utils::server_fn_error!("Expected data, got message")
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}
