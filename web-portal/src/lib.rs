use std::fmt::Display;

use actix_multipart::form::{text::Text, MultipartForm};
use actix_session::Session;
use actix_web::{http::header::ContentType, HttpResponse};
use askama::Template;
use common::api::ApiResponseBody;
use reqwest::{Client, IntoUrl, Method, Response, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use users::data::user::User;
use uuid::Uuid;

const SESSION_KEY: &str = "em_uid";
const INTERNAL_SERVICE_ERROR: &str = "Error contacting internal service";

mod utils {
    macro_rules! server_fn_error {
        ($f:literal, $($item:ident)+) => {
            Err(ServerFnError::Generic(format!($f, $($item)+)))
        };
    }

    macro_rules! internal_server_error {
        ($error:ident) => {
            HttpResponse::InternalServerError().body(format!("{}", $error))
        };
        ($t:literal) => {
            HttpResponse::InternalServerError().body($t)
        };
        () => {
            HttpResponse::InternalServerError()
                .body("Error within the server that cannot be recovered. Contact administrator")
        };
    }

    macro_rules! redirect {
        ($location:literal) => {
            HttpResponse::Found()
                .insert_header(("location", $location))
                .finish()
        };
    }

    macro_rules! html_response {
        ($html:ident) => {
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body($html)
        };
    }

    macro_rules! text {
        ($text:literal) => {
            HttpResponse::Ok().body($text)
        };
    }
    pub(crate) use html_response;
    pub(crate) use internal_server_error;
    pub(crate) use redirect;
    pub(crate) use server_fn_error;
    pub(crate) use text;
}

#[derive(Debug, Error)]
pub enum ServerFnError {
    #[error(transparent)]
    Serialization(#[from] rmp_serde::encode::Error),
    #[error(transparent)]
    Deserialization(#[from] rmp_serde::decode::Error),
    #[error("Error performing API request. {0}")]
    ApiRequest(reqwest::Error),
    #[error("Invalid API response: {0}")]
    ApiResponse(StatusCode),
    #[error("Api response body cannot be processed. {0}")]
    ApiResponseBody(reqwest::Error),
    #[error("{0}")]
    Generic(String),
}

fn validate_session(session: Session) -> Option<Uuid> {
    match session.get(SESSION_KEY) {
        Ok(result) => result,
        Err(error) => {
            log::error!("{error}");
            None
        }
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

pub async fn login(session: Session) -> HttpResponse {
    if validate_session(session).is_some() {
        return utils::redirect!("/");
    }
    let html = match LoginTemplate.render() {
        Ok(inner) => inner,
        Err(error) => return utils::internal_server_error!(error),
    };
    utils::html_response!(html)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub async fn index(session: Session) -> HttpResponse {
    let Some(_) = validate_session(session) else {
        return utils::redirect!("/login");
    };
    let html = match IndexTemplate.render() {
        Ok(inner) => inner,
        Err(error) => return utils::internal_server_error!(error),
    };
    utils::html_response!(html)
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
            return utils::server_fn_error!("Server error. {}", message)
        }
    };
    Ok(Some(user))
}
