pub mod api;
pub mod pages;

use actix_session::Session;
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
            Err(ServerFnError::Generic($item.to_owned()))
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

    macro_rules! text {
        ($text:literal) => {
            HttpResponse::Ok().body($text)
        };
    }
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
