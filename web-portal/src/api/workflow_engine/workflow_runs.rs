use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use workflow_engine::workflow_run::data::{WorkflowRun, WorkflowRunId};

use crate::{
    components::{ActiveWorkflowRuns, ActiveWorkflowRunsTab},
    extract_session_uid, utils, ServerFnError,
};

pub fn service() -> actix_web::Scope {
    web::scope("/workflow-runs")
        .route("/tab", web::get().to(active_workflow_runs_tab))
        .route(
            "/schedule/{workflow_run_id}",
            web::post().to(schedule_workflow_run),
        )
        .route(
            "/cancel/{workflow_run_id}",
            web::post().to(cancel_workflow_run),
        )
        .route(
            "/restart/{workflow_run_id}",
            web::post().to(restart_workflow_run),
        )
}

async fn active_workflow_runs(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    let workflow_runs = match get_active_workflow_runs().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <ActiveWorkflowRuns workflow_runs=workflow_runs /> }
    });
    utils::html_chunk!(html)
}

async fn active_workflow_runs_tab(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    let workflow_runs = match get_active_workflow_runs().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <ActiveWorkflowRunsTab workflow_runs=workflow_runs /> }
    });
    utils::html_chunk!(html)
}

async fn get_active_workflow_runs() -> Result<Vec<WorkflowRun>, ServerFnError> {
    let workflow_runs_response = utils::api_request(
        "http://127.0.0.1:8000/api/v1/workflow-runs?f=msgpack",
        Method::GET,
        None::<String>,
        None::<()>,
    )
    .await?;
    let executors = match workflow_runs_response {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            return utils::server_fn_error!(message)
        }
    };
    Ok(executors)
}

async fn schedule_workflow_run(
    session: Session,
    workflow_run_id: web::Path<WorkflowRunId>,
) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    if let Err(error) = post_schedule_workflow_run(workflow_run_id.into_inner()).await {
        return error.to_response();
    }
    active_workflow_runs(session).await
}

async fn post_schedule_workflow_run(workflow_run_id: WorkflowRunId) -> Result<(), ServerFnError> {
    let clean_executors_response: ApiResponseBody<WorkflowRun> = utils::api_request(
        format!("http://127.0.0.1:8000/api/v1/workflow-runs/schedule/{workflow_run_id}?f=msgpack"),
        Method::POST,
        None::<String>,
        None::<()>,
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(workflow_run) => {
            log::info!("Scheduled workflow run: {}", workflow_run.workflow_run_id);
            Ok(())
        }
        ApiResponseBody::Message(message) => {
            utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}

async fn cancel_workflow_run(
    session: Session,
    workflow_run_id: web::Path<WorkflowRunId>,
) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    if let Err(error) = post_cancel_workflow_run(workflow_run_id.into_inner()).await {
        return error.to_response();
    }
    active_workflow_runs(session).await
}

async fn post_cancel_workflow_run(workflow_run_id: WorkflowRunId) -> Result<(), ServerFnError> {
    let clean_executors_response: ApiResponseBody<WorkflowRun> = utils::api_request(
        format!("http://127.0.0.1:8000/api/v1/workflow-runs/cancel/{workflow_run_id}?f=msgpack"),
        Method::POST,
        None::<String>,
        None::<()>,
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(workflow_run) => {
            log::info!("Canceled workflow run: {}", workflow_run.workflow_run_id);
            Ok(())
        }
        ApiResponseBody::Message(message) => {
            utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}

async fn restart_workflow_run(
    session: Session,
    workflow_run_id: web::Path<WorkflowRunId>,
) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login_htmx!();
    }
    if let Err(error) = post_restart_workflow_run(workflow_run_id.into_inner()).await {
        return error.to_response();
    }
    active_workflow_runs(session).await
}

async fn post_restart_workflow_run(workflow_run_id: WorkflowRunId) -> Result<(), ServerFnError> {
    let clean_executors_response: ApiResponseBody<WorkflowRun> = utils::api_request(
        format!("http://127.0.0.1:8000/api/v1/workflow-runs/restart/{workflow_run_id}?f=msgpack"),
        Method::POST,
        None::<String>,
        None::<()>,
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(workflow_run) => {
            log::info!("Restarted workflow run: {}", workflow_run.workflow_run_id);
            Ok(())
        }
        ApiResponseBody::Message(message) => {
            utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}
