use std::fmt::Display;

use actix_multipart::form::{text::Text, MultipartForm};
use actix_session::Session;
use actix_web::HttpResponse;
use common::api::ApiResponseBody;
use reqwest::{Client, IntoUrl, Method, Response};
use serde::{Deserialize, Serialize};
use users::data::user::User;
use uuid::Uuid;

use crate::{utils, ServerFnError, INTERNAL_SERVICE_ERROR, SESSION_KEY};

async fn send_request<U, D, T>(
    url: U,
    method: Method,
    auth: Option<D>,
    body: Option<T>,
) -> Result<Response, ServerFnError>
where
    U: IntoUrl,
    D: Display,
    T: Serialize,
{
    let client = Client::new();
    let mut builder = client.request(method, url);
    if let Some(auth) = auth {
        builder = builder.header("Authorization", format!("Bearer {auth}"))
    }
    if let Some(body) = body {
        let body = rmp_serde::to_vec(&body)?;
        builder = builder
            .body(body)
            .header("Content-Type", "application/msgpack")
    }
    let response = builder.send().await.map_err(ServerFnError::ApiRequest)?;
    Ok(response)
}

async fn process_response<T>(response: Response) -> Result<ApiResponseBody<T>, ServerFnError>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    if !response.status().is_success() {
        return Err(ServerFnError::ApiResponse(response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(ServerFnError::ApiResponseBody)?;
    let data = rmp_serde::from_slice::<ApiResponseBody<T>>(&bytes)?;
    Ok(data)
}

#[derive(MultipartForm)]
pub struct CredentialsFormData {
    username: Text<String>,
    password: Text<String>,
}

impl From<CredentialsFormData> for Credentials {
    fn from(val: CredentialsFormData) -> Self {
        Credentials {
            username: val.username.0,
            password: val.password.0,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Credentials {
    username: String,
    password: String,
}

pub async fn logout_user(session: Option<Session>) -> HttpResponse {
    if let Some(session) = session {
        session.clear()
    }
    utils::redirect!("/login")
}

pub async fn login_user(
    session: Session,
    credentials: MultipartForm<CredentialsFormData>,
) -> HttpResponse {
    let user = match login_user_api(credentials.0.into()).await {
        Ok(Some(inner)) => inner,
        Ok(None) => return utils::text!("User validation failed"),
        Err(error) => {
            log::error!("{error}");
            return utils::internal_server_error!(INTERNAL_SERVICE_ERROR);
        }
    };
    if let Err(error) = session.insert(SESSION_KEY, *user.uid()) {
        log::error!("{error}");
        return utils::internal_server_error!("Error trying to create a new session for the user");
    }
    utils::redirect!("/")
}

async fn login_user_api(credentials: Credentials) -> Result<Option<User>, ServerFnError> {
    let login_response = send_request(
        "http://127.0.0.1:8001/api/v1/users/validate?f=msgpack",
        Method::POST,
        None::<String>,
        Some(credentials),
    )
    .await?;
    let user = match process_response::<User>(login_response).await? {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Failure(_) => return Ok(None),
        ApiResponseBody::Error(message) => {
            log::error!("{message}");
            return utils::server_fn_error!(INTERNAL_SERVICE_ERROR);
        }
    };
    Ok(Some(user))
}

pub async fn get_user(uid: Uuid) -> Result<User, ServerFnError> {
    let user_response = send_request(
        "http://127.0.0.1:8001/api/v1/user?f=msgpack",
        Method::GET,
        Some(uid),
        None::<()>,
    )
    .await?;
    let user = match process_response::<User>(user_response).await? {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            log::error!("{message}");
            return utils::server_fn_error!(INTERNAL_SERVICE_ERROR);
        }
    };
    Ok(user)
}
