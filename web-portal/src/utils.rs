use std::fmt::Display;

use actix_session::Session;
use actix_web::{HttpResponse, HttpResponseBuilder};
use common::api::ApiResponseBody;
use reqwest::{Client, IntoUrl, Method, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use users::data::user::User;
use uuid::Uuid;

use crate::{components::modal::MODAL_ERROR_MESSAGE_ID, extract_session_uid, ServerFnError};

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
        let status_code = response.status();
        let text = match response.text().await {
            Ok(inner) => Some(inner),
            Err(error) => {
                log::error!("{error}");
                None
            }
        };
        return Err(ServerFnError::ApiResponse(status_code, text));
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

pub async fn get_user_session(session: Session) -> Result<User, ServerFnError> {
    let uid = extract_session_uid(&session)?;
    get_user(uid, None).await
}

pub async fn get_user(current_uid: Uuid, other_uid: Option<Uuid>) -> Result<User, ServerFnError> {
    let url = match other_uid {
        Some(uid) => format!("http://127.0.0.1:8001/api/v1/user/{uid}?f=msgpack"),
        None => "http://127.0.0.1:8001/api/v1/user?f=msgpack".to_owned(),
    };
    let user_response = api_request(url, Method::GET, Some(current_uid), None::<()>).await?;
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

pub const HOME_LOCATION: &str = "/";
pub const LOGIN_LOCATION: &str = "/login";

pub struct HtmxResponseBuilder {
    response: HttpResponseBuilder,
    triggers: Option<Vec<(&'static str, serde_json::Value)>>,
}

impl Default for HtmxResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmxResponseBuilder {
    pub fn new() -> Self {
        let mut response = HttpResponse::Ok();
        response.content_type(actix_web::http::header::ContentType::html());
        Self {
            response,
            triggers: None,
        }
    }

    pub fn add_close_modal_event<S>(&mut self, modal_id: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.add_trigger_event("closeModal", json!({"id": modal_id.as_ref()}))
    }

    pub fn add_create_toast_event<S>(&mut self, message: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.add_trigger_event("createToast", json!({"message": message.as_ref()}))
    }

    pub fn add_trigger_event(&mut self, event: &'static str, data: serde_json::Value) -> &mut Self {
        match self.triggers.as_mut() {
            Some(triggers) => triggers.push((event, data)),
            None => self.triggers = Some(vec![(event, data)]),
        };
        self
    }

    pub fn redirect_home() -> HttpResponse {
        Self::redirect("/")
    }

    pub fn redirect_login() -> HttpResponse {
        Self::redirect("/login")
    }

    pub fn redirect<S>(location: S) -> HttpResponse
    where
        S: AsRef<str>,
    {
        HttpResponse::Found()
            .insert_header(("HX-Redirect", location.as_ref()))
            .finish()
    }

    pub fn target<S>(&mut self, target: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.response
            .insert_header(("HX-Retarget", target.as_ref()));
        self
    }

    pub fn swap<S>(&mut self, swap: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.response.insert_header(("HX-Swap", swap.as_ref()));
        self
    }

    fn finish_triggers(&mut self) -> &mut Self {
        let triggers_option = self.triggers.take();
        match triggers_option {
            Some(triggers) if !triggers.is_empty() => {
                let data = triggers
                    .into_iter()
                    .map(|(key, obj)| format!("\"{key}\": {obj}"))
                    .collect::<Vec<String>>()
                    .join(",");
                log::info!("HX-Trigger {{{data}}}");
                self.response
                    .insert_header(("HX-Trigger", format!("{{{data}}}")));
            }
            _ => {}
        }
        self
    }

    pub fn modal_error_message<S>(message: S) -> HttpResponse
    where
        S: Into<String>,
    {
        Self::new()
            .target(format!("#{MODAL_ERROR_MESSAGE_ID}"))
            .swap("innerHTML")
            .raw_body(message.into())
    }

    pub fn static_body(&mut self, html: &'static str) -> HttpResponse {
        self.finish_triggers();
        self.response.body(html)
    }

    pub fn raw_body(&mut self, html: String) -> HttpResponse {
        self.finish_triggers();
        self.response.body(html)
    }

    pub fn html_chunk<F, IV>(&mut self, html: F) -> HttpResponse
    where
        F: FnOnce(leptos::Scope) -> IV + 'static,
        IV: leptos::IntoView,
    {
        self.finish_triggers();
        let html = leptos::ssr::render_to_string(html);
        self.response.body(html)
    }

    pub fn location_home() -> HttpResponse {
        Self::location(HOME_LOCATION)
    }

    pub fn location_login() -> HttpResponse {
        Self::location(LOGIN_LOCATION)
    }

    pub fn location<S>(location: S) -> HttpResponse
    where
        S: Into<String>,
    {
        let mut builder = Self::new();
        builder
            .response
            .insert_header(("HX-Location", location.into()))
            .finish()
    }
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

pub fn html_page<F, IV>(html: F) -> HttpResponse
where
    F: FnOnce(leptos::Scope) -> IV + 'static,
    IV: leptos::IntoView,
{
    let mut html = leptos::ssr::render_to_string(html);
    html.insert_str(0, "<!DOCTYPE html>");
    HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::html())
        .body(html)
}

macro_rules! redirect_home {
    () => {
        HttpResponse::Found()
            .insert_header(("location", "/"))
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

macro_rules! redirect_login_htmx {
    () => {
        HttpResponse::Found()
            .insert_header(("HX-Redirect", "/login"))
            .finish()
    };
}

pub(crate) use internal_server_error;
pub(crate) use redirect_home;
pub(crate) use redirect_login;
pub(crate) use redirect_login_htmx;
pub(crate) use server_fn_error;
pub(crate) use server_fn_static_error;
