pub mod api;
pub mod components;
pub mod pages;

use actix_session::Session;
use actix_web::HttpResponse;
use common::api::{ApiContentFormat, ApiResponse};
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

pub const EM_UID_SESSION_KEY: &str = "em_uid";
pub const USERNAME_SESSION_KEY: &str = "username";
pub const INTERNAL_SERVICE_ERROR: &str = "Error contacting internal service";

pub mod utils;

#[derive(Debug, Error)]
pub enum ServerFnError {
    #[error(transparent)]
    Serialization(#[from] rmp_serde::encode::Error),
    #[error(transparent)]
    Deserialization(#[from] rmp_serde::decode::Error),
    #[error("Error performing API request. {0}")]
    ApiRequest(reqwest::Error),
    #[error("Invalid API response: {0}. {1:?}")]
    ApiResponse(StatusCode, Option<String>),
    #[error("Api response body cannot be processed. {0}")]
    ApiResponseBody(reqwest::Error),
    #[error(transparent)]
    Session(#[from] actix_session::SessionGetError),
    #[error("User attempted to access endpoint without a valid session")]
    InvalidUser,
    #[error("{0}")]
    Generic(String),
    #[error("{0}")]
    StaticGeneric(&'static str),
}

impl ServerFnError {
    pub fn to_api_response<T>(self, format: ApiContentFormat) -> ApiResponse<T>
    where
        T: Serialize,
    {
        log::error!("{}", self);
        ApiResponse::failure("Error during internal API request", format)
    }

    pub fn to_response(&self) -> HttpResponse {
        log::error!("{}", self);
        utils::internal_server_error!()
    }
}

fn extract_session_uid(session: &Session) -> Result<Uuid, ServerFnError> {
    let Some(user) = session.get(EM_UID_SESSION_KEY)? else {
        return Err(ServerFnError::InvalidUser);
    };
    Ok(user)
}

fn take_if<T, F>(value: T, predicate: F) -> Option<T>
where
    F: FnOnce(&T) -> bool,
{
    if predicate(&value) {
        return Some(value);
    }
    None
}

fn error_if<T, F, E, M>(value: T, predicate: F, error_message: M) -> Result<T, E>
where
    F: FnOnce(&T) -> bool,
    M: FnOnce(&T) -> E,
{
    if predicate(&value) {
        return Err(error_message(&value));
    }
    Ok(value)
}
