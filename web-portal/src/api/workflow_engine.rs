use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use workflow_engine::{executor::data::Executor, workflow_run::data::WorkflowRun};

use crate::{
    components::{ActiveExecutors, ActiveWorkflowRuns},
    utils,
    utils::api_request,
    validate_session, ServerFnError,
};

pub fn service() -> actix_web::Scope {
    web::scope("/workflow-engine")
        .route("/executors", web::get().to(active_executors))
        .route("/executors/clean", web::post().to(clean_executors))
        .route("/workflow-runs", web::get().to(active_workflow_runs))
}

async fn active_executors(session: Session) -> HttpResponse {
    if validate_session(&session).is_err() {
        return utils::redirect_login!();
    }
    let executors = match get_active_executors().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <ActiveExecutors executors=executors /> }
    });
    utils::html_chunk!(html)
}

async fn get_active_executors() -> Result<Vec<Executor>, ServerFnError> {
    let executors_response = api_request(
        "http://127.0.0.1:8000/api/v1/executors?f=msgpack",
        Method::GET,
        None::<String>,
        None::<()>,
    )
    .await?;
    let executors = match executors_response {
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

async fn active_workflow_runs(session: Session) -> HttpResponse {
    if validate_session(&session).is_err() {
        return utils::redirect_login!();
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

async fn get_active_workflow_runs() -> Result<Vec<WorkflowRun>, ServerFnError> {
    let workflow_runs_response = api_request(
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

async fn clean_executors(session: Session) -> HttpResponse {
    if validate_session(&session).is_err() {
        return utils::redirect_login!();
    }
    if let Err(error) = post_clean_executors().await {
        return error.to_response();
    }
    active_executors(session).await
}

async fn post_clean_executors() -> Result<(), ServerFnError> {
    let clean_executors_response = api_request(
        "http://127.0.0.1:8000/api/v1/executors/clean?f=msgpack",
        Method::POST,
        None::<String>,
        None::<()>,
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(()) => {
            utils::server_fn_error!("Expected message, got data")
        }
        ApiResponseBody::Message(message) => {
            log::info!("{message}");
            Ok(())
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}
