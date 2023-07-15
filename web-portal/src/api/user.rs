use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use users::{data::user::User, service::users::ValidateUserRequest};

use crate::{
    api,
    auth::{get_uid, set_session},
};

const SESSION_KEY: &str = "em_uid";

#[server(LoginUser, "/api")]
pub async fn login_user(
    cx: Scope,
    username: String,
    password: String,
) -> Result<(), ServerFnError> {
    let credentials = ValidateUserRequest::new(username, password);
    let login_response = api::send_request(
        "http://127.0.0.1:8001/api/v1/users/validate?f=msgpack",
        Method::POST,
        None::<String>,
        Some(credentials),
    )
    .await?;
    let user = match api::process_response::<User>(login_response).await? {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return api::utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Failure(_) => return Ok(()),
        ApiResponseBody::Error(message) => {
            return api::utils::server_fn_error!("Server error. {}", message)
        }
    };
    let session = leptos_actix::extract(cx, |session: actix_session::Session| async move {
        log::info!("Session {:?}", session.entries());
        session
    })
    .await?;
    session
        .insert(SESSION_KEY, *user.uid())
        .map_err(|e| ServerFnError::ServerError(format!("Could not create a new session. {e}")))?;
    // set_session(cx, *user.uid()).await?;

    leptos_actix::redirect(cx, "/");
    Ok(())
}

#[server(GetUser, "/api")]
pub async fn get_user(cx: Scope) -> Result<Option<User>, ServerFnError> {
    let uid = match get_uid(cx).await {
        Ok(inner) => inner,
        Err(error) => {
            if let ServerFnError::Request(_) = error {
                log::warn!("User not authenticated");
            } else {
                log::error!("{}", error);
            }
            return Ok(None);
        }
    };
    let user_response = api::send_request(
        "http://127.0.0.1:8001/api/v1/user?f=msgpack",
        Method::GET,
        Some(uid),
        None::<()>,
    )
    .await?;
    let user = match api::process_response::<User>(user_response).await? {
        ApiResponseBody::Success(inner) => Some(inner),
        ApiResponseBody::Message(message) | ApiResponseBody::Failure(message) => {
            warn!("Could not get user details. {}", message);
            None
        }
        ApiResponseBody::Error(message) => {
            return api::utils::server_fn_error!("Server error. {}", message)
        }
    };
    Ok(user)
}
