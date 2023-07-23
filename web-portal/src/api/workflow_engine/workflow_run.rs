use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use workflow_engine::workflow_run::data::{WorkflowRun, WorkflowRunId};

use crate::{components::WorkflowRunDisplay, extract_session_uid, utils, ServerFnError};

pub fn service() -> actix_web::Scope {
    web::scope("/workflow-run").service(
        web::resource("/{workflow_run_id}")
            .route(web::post().to(enter_workflow_run))
            .route(web::get().to(workflow_run)),
    )
}

async fn enter_workflow_run(
    session: Session,
    workflow_run_id: web::Path<WorkflowRunId>,
) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    let workflow_run_id = workflow_run_id.into_inner();
    utils::redirect_htmx!("/workflow-engine/workflow-run/{}", workflow_run_id)
}

async fn workflow_run(session: Session, workflow_run_id: web::Path<WorkflowRunId>) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    let workflow_run_id = workflow_run_id.into_inner();
    let workflow_run = match get_workflow_run(workflow_run_id).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <WorkflowRunDisplay workflow_run=workflow_run/> }
    });
    utils::html_chunk!(html)
}

pub async fn get_workflow_run(
    workflow_run_id: WorkflowRunId,
) -> Result<WorkflowRun, ServerFnError> {
    let clean_executors_response: ApiResponseBody<WorkflowRun> = utils::api_request(
        format!("http://127.0.0.1:8000/api/v1/workflow-runs/{workflow_run_id}?f=msgpack"),
        Method::GET,
        None::<String>,
        None::<()>,
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(workflow_run) => Ok(workflow_run),
        ApiResponseBody::Message(message) => {
            utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}
