use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
// use serde::{Deserialize, Serialize};
use users::data::user::User;
use uuid::Uuid;

use crate::{components::UsersTable, extract_session_uid, utils, ServerFnError};

pub fn service() -> actix_web::Scope {
    web::scope("/users").route("", web::get().to(all_users))
}

async fn all_users(session: Session) -> HttpResponse {
    let Ok(uid) = extract_session_uid(&session) else {
        return utils::redirect_login_htmx!();
    };
    let users = match get_all_users(uid).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <UsersTable uid=uid users=users/> }
    });
    utils::html_chunk!(html)
}

pub async fn get_all_users(uid: Uuid) -> Result<Vec<User>, ServerFnError> {
    let executors_response = utils::api_request(
        "http://127.0.0.1:8001/api/v1/users?f=msgpack",
        Method::GET,
        Some(uid),
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
