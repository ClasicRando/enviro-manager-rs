pub mod api;
pub mod pages;

use actix_session::Session;
use actix_web::HttpResponse;
use reqwest::StatusCode;
use thiserror::Error;
use uuid::Uuid;

pub const SESSION_KEY: &str = "em_uid";
pub const INTERNAL_SERVICE_ERROR: &str = "Error contacting internal service";

pub mod utils {
    macro_rules! server_fn_error {
        ($f:literal, $($item:ident)+) => {
            Err(ServerFnError::Generic(format!($f, $($item)+)))
        };
        ($item:ident) => {
            Err(ServerFnError::Generic($item))
        };
    }
    macro_rules! server_fn_static_error {
        ($item:ident) => {
            Err(ServerFnError::StaticGeneric($item))
        };
        ($item:literal) => {
            Err(ServerFnError::StaticGeneric($item))
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

    #[macro_export]
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

    macro_rules! json {
        ($data:ident) => {
            HttpResponse::Ok().json($data)
        };
    }

    pub(crate) use internal_server_error;
    pub(crate) use json;
    pub(crate) use redirect;
    pub use redirect_home;
    pub(crate) use redirect_login;
    pub(crate) use server_fn_error;
    pub(crate) use server_fn_static_error;
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
    pub fn to_response(&self) -> HttpResponse {
        log::error!("{}", self);
        utils::internal_server_error!()
    }
}

fn validate_session(session: Session) -> Result<Uuid, ServerFnError> {
    let Some(user) = session.get(SESSION_KEY)? else {
        return Err(ServerFnError::InvalidUser)
    };
    Ok(user)
}
