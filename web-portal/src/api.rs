use std::fmt::Display;

use actix_multipart::form::{text::Text, MultipartForm};
use actix_session::Session;
use actix_web::HttpResponse;
use common::api::{ApiContentFormat, ApiResponse, ApiResponseBody};
use reqwest::{Client, IntoUrl, Method, Response};
use serde::{Deserialize, Serialize};
use users::data::user::User;
use workflow_engine::{executor::data::Executor, workflow_run::data::WorkflowRun};

use crate::{
    components, utils, validate_session, ServerFnError, INTERNAL_SERVICE_ERROR, SESSION_KEY,
};

macro_rules! invalid_user_api_response {
    () => {
        ApiResponse::failure("User is not validated", ApiContentFormat::Json)
    };
}

macro_rules! json_api_success {
    (()) => {
        ApiResponse::success((), ApiContentFormat::Json)
    };
    ($data:ident) => {
        ApiResponse::success($data, ApiContentFormat::Json)
    };
}

macro_rules! json_api_failure {
    ($message:literal) => {
        ApiResponse::failure($message, ApiContentFormat::Json)
    };
}

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

async fn api_request<U, D, B, T>(
    url: U,
    method: Method,
    auth: Option<D>,
    body: Option<B>,
) -> Result<ApiResponseBody<T>, ServerFnError>
where
    U: IntoUrl,
    D: Display,
    B: Serialize,
    T: Serialize + for<'de> Deserialize<'de>,
{
    let response = send_request(url, method, auth, body).await?;
    let data = process_response::<T>(response).await?;
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
) -> ApiResponse<()> {
    let user = match login_user_api(credentials.0.into()).await {
        Ok(inner) => inner,
        Err(error) => return error.to_api_response(ApiContentFormat::Json),
    };
    if let Err(error) = session.insert(SESSION_KEY, *user.uid()) {
        log::error!("{error}");
        return json_api_failure!("Could not insert user session");
    }
    json_api_success!(())
}

async fn login_user_api(credentials: Credentials) -> Result<User, ServerFnError> {
    let user_response = api_request(
        "http://127.0.0.1:8001/api/v1/users/validate?f=msgpack",
        Method::POST,
        None::<String>,
        Some(credentials),
    )
    .await?;
    let user = match user_response {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            log::error!("Expected data, got message. {message}");
            return utils::server_fn_error!("Expected data, got message. {}", message);
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            log::error!("{message}");
            return utils::server_fn_static_error!(INTERNAL_SERVICE_ERROR);
        }
    };
    Ok(user)
}

pub async fn get_user(session: Session) -> Result<User, ServerFnError> {
    let uid = validate_session(session)?;
    let user_response = api_request(
        "http://127.0.0.1:8001/api/v1/user?f=msgpack",
        Method::GET,
        Some(uid),
        None::<()>,
    )
    .await?;
    let user = match user_response {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            return utils::server_fn_error!(message)
        }
    };
    Ok(user)
}

pub async fn active_executors(session: Session) -> ApiResponse<Vec<Executor>> {
    if validate_session(session).is_err() {
        return invalid_user_api_response!();
    }
    let executors = match get_active_executors().await {
        Ok(inner) => inner,
        Err(error) => return error.to_api_response(ApiContentFormat::Json),
    };
    json_api_success!(executors)
}

pub async fn active_executors_html(session: Session) -> HttpResponse {
    if validate_session(session).is_err() {
        return utils::redirect_login!();
    }
    let executors = match get_active_executors().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = components::active_executors(executors).0;
    utils::html!(html)
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

pub async fn active_workflow_runs_html(session: Session) -> HttpResponse {
    if validate_session(session).is_err() {
        return utils::redirect_login!();
    }
    let workflow_runs = match get_active_workflow_runs().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = components::active_workflow_runs(workflow_runs).0;
    utils::html!(html)
}

pub async fn active_workflow_runs(session: Session) -> ApiResponse<Vec<WorkflowRun>> {
    if validate_session(session).is_err() {
        return invalid_user_api_response!();
    }
    let workflow_runs = match get_active_workflow_runs().await {
        Ok(inner) => inner,
        Err(error) => return error.to_api_response(ApiContentFormat::Json),
    };
    json_api_success!(workflow_runs)
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
