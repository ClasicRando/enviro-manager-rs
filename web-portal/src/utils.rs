use std::fmt::Display;

use actix_session::Session;
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::{Client, IntoUrl, Method, Response};
use serde::{Deserialize, Serialize};
use users::data::user::User;

use crate::{validate_session, ServerFnError};

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

pub async fn api_request<U, D, B, T>(
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

pub async fn get_user(session: Session) -> Result<User, ServerFnError> {
    let uid = validate_session(&session)?;
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
            return server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            return server_fn_error!(message)
        }
    };
    Ok(user)
}

macro_rules! server_fn_error {
    ($f:literal, $($item:ident)+) => {
        Err(crate::ServerFnError::Generic(format!($f, $($item)+)))
    };
    ($item:ident) => {
        Err(crate::ServerFnError::Generic($item))
    };
    ($string:literal) => {
        Err(crate::ServerFnError::Generic($string.to_owned()))
    };
}
macro_rules! server_fn_static_error {
    ($item:ident) => {
        Err(crate::ServerFnError::StaticGeneric($item))
    };
    ($item:literal) => {
        Err(crate::ServerFnError::StaticGeneric($item))
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

macro_rules! html {
    ($html:ident) => {{
        $html.insert_str(0, "<!DOCTYPE html>");
        HttpResponse::Ok()
            .content_type(actix_web::http::header::ContentType::html())
            .body($html)
    }};
}

macro_rules! html_chunk {
    ($html:ident) => {
        HttpResponse::Ok()
            .content_type(actix_web::http::header::ContentType::html())
            .body($html)
    };
    ($html:literal) => {
        HttpResponse::Ok()
            .content_type(actix_web::http::header::ContentType::html())
            .body($html)
    };
}

macro_rules! redirect {
    ($location:literal) => {
        HttpResponse::Found()
            .insert_header(("location", $location))
            .finish()
    };
}

macro_rules! redirect_home {
    () => {
        HttpResponse::Found()
            .insert_header(("location", "/"))
            .finish()
    };
}

macro_rules! redirect_home_htmx {
    () => {
        HttpResponse::Found()
            .insert_header(("HX-Redirect", "/"))
            .finish()
    };
}

macro_rules! redirect_login {
    () => {
        HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish()
    };
}

pub(crate) use html;
pub(crate) use html_chunk;
pub(crate) use internal_server_error;
pub(crate) use redirect;
pub(crate) use redirect_home;
pub(crate) use redirect_home_htmx;
pub(crate) use redirect_login;
pub(crate) use server_fn_error;
pub(crate) use server_fn_static_error;
