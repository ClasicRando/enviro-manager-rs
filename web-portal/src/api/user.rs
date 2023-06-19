use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use users::{data::user::User, service::users::ValidateUserRequest};

use crate::{
    api,
    auth::{get_uid, set_session},
};

#[server(LoginUser, "/api/login")]
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
        ApiResponseBody::Failure(message) => {
            return api::utils::server_fn_error!("Server failure. {}", message)
        }
        ApiResponseBody::Error(message) => {
            return api::utils::server_fn_error!("Server error. {}", message)
        }
    };
    set_session(cx, *user.uid()).await?;

    leptos_actix::redirect(cx, "/");
    Ok(())
}

#[server(GetUser)]
pub async fn get_user(cx: Scope) -> Result<User, ServerFnError> {
    let uid = get_uid(cx).await?;
    let user_response = api::send_request(
        "http://127.0.0.1:8001/api/v1/user?f=msgpack",
        Method::GET,
        Some(uid),
        None::<()>,
    )
    .await?;
    let user = match api::process_response::<User>(user_response).await? {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return api::utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Failure(message) => {
            return api::utils::server_fn_error!("Server failure. {}", message)
        }
        ApiResponseBody::Error(message) => {
            return api::utils::server_fn_error!("Server error. {}", message)
        }
    };
    Ok(user)
}
