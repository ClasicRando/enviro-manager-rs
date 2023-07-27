use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use workflow_engine::executor::data::{Executor, ExecutorId};

use crate::{
    components::workflow_engine::main_page::{ActiveExecutors, ActiveExecutorsTab},
    extract_session_uid, utils,
    utils::HtmxResponseBuilder,
    ServerFnError,
};

pub fn service() -> actix_web::Scope {
    web::scope("/executors")
        .route("", web::get().to(active_executors))
        .route("/tab", web::get().to(active_executors_tab))
        .route("/clean", web::post().to(clean_executors))
        .route("/cancel/{executor_id}", web::post().to(cancel_executor))
        .route("/shutdown/{executor_id}", web::post().to(shutdown_executor))
}

async fn active_executors_html(is_tab: bool) -> HttpResponse {
    active_executors_html_with_toast(is_tab, "").await
}

async fn active_executors_html_with_toast<S>(is_tab: bool, toast_message: S) -> HttpResponse
where
    S: AsRef<str>,
{
    let executors = match get_active_executors().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };

    let mut builder = HtmxResponseBuilder::new();
    if !toast_message.as_ref().is_empty() {
        builder.add_create_toast_event(toast_message.as_ref());
    }
    builder.html_chunk(move |cx| {
        if is_tab {
            view! { cx, <ActiveExecutorsTab executors=executors/> }
        } else {
            view! { cx, <ActiveExecutors executors=executors/> }
        }
    })
}

async fn active_executors(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }
    active_executors_html(false).await
}

async fn active_executors_tab(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }
    active_executors_html(true).await
}

async fn get_active_executors() -> Result<Vec<Executor>, ServerFnError> {
    let executors_response = utils::api_request(
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

async fn clean_executors(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }
    if let Err(error) = post_clean_executors().await {
        return error.to_response();
    }

    active_executors_html_with_toast(false, "Cleaned inactive executors").await
}

async fn post_clean_executors() -> Result<(), ServerFnError> {
    let clean_executors_response = utils::api_request(
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

async fn cancel_executor(session: Session, executor_id: web::Path<ExecutorId>) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }
    let executor_id = executor_id.into_inner();
    if let Err(error) = post_cancel_executor(executor_id).await {
        return error.to_response();
    }

    active_executors_html_with_toast(false, format!("Canceled Executor ID: {executor_id}")).await
}

async fn post_cancel_executor(executor_id: ExecutorId) -> Result<(), ServerFnError> {
    let clean_executors_response: ApiResponseBody<Executor> = utils::api_request(
        format!("http://127.0.0.1:8000/api/v1/executors/cancel/{executor_id}?f=msgpack"),
        Method::POST,
        None::<String>,
        None::<()>,
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(executor) => {
            log::info!("Canceled executor: {}", executor.executor_id);
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

async fn shutdown_executor(session: Session, executor_id: web::Path<ExecutorId>) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }
    let executor_id = executor_id.into_inner();
    if let Err(error) = post_shutdown_executor(executor_id).await {
        return error.to_response();
    }

    active_executors_html_with_toast(false, format!("Shutdown Executor ID: {executor_id}")).await
}

async fn post_shutdown_executor(executor_id: ExecutorId) -> Result<(), ServerFnError> {
    let clean_executors_response: ApiResponseBody<Executor> = utils::api_request(
        format!("http://127.0.0.1:8000/api/v1/executors/shutdown/{executor_id}?f=msgpack"),
        Method::POST,
        None::<String>,
        None::<()>,
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(executor) => {
            log::info!("Canceled executor: {}", executor.executor_id);
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
