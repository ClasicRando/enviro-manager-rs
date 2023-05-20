use std::error::Error;

use actix_web::Responder;
use log::{error, warn};
use serde::Serialize;

/// Generic response object as an API response. A response is either a success containing data, a
/// message to let the user know what happened or an error/failure message.
#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ApiResponse<T: Serialize> {
    Success(T),
    Message(String),
    Error(String),
}

impl<T> Responder for ApiResponse<T>
where
    T: Serialize + 'static,
{
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let bytes = match rmp_serde::to_vec(&self) {
            Ok(inner) => inner,
            Err(error) => {
                let message = format!(
                    "Could not serialize response for {}. Error: {}",
                    req.path(),
                    error
                );
                error!("{}", message);
                return actix_web::HttpResponse::InternalServerError()
                    .content_type(actix_web::http::header::ContentType::plaintext())
                    .body(message.into_bytes());
            }
        };

        actix_web::HttpResponse::Ok()
            .content_type(actix_web::http::header::ContentType(
                mime::APPLICATION_MSGPACK,
            ))
            .body(actix_web::web::Bytes::from_iter(bytes.into_iter()))
    }
}

impl<T: Serialize> ApiResponse<T> {
    /// Generate an [ApiResponse] wrapping a [Response::Success]`
    pub const fn success(data: T) -> Self {
        Self::Success(data)
    }

    /// Generate an [ApiResponse] wrapping a [Response::Message]
    pub const fn message(message: String) -> Self {
        Self::Message(message)
    }

    /// Generate an [ApiResponse] wrapping a [Response::Error]. This is intended for errors that
    /// are not runtime errors but rather user input issues.
    pub fn failure(message: String) -> Self {
        warn!("{}", message);
        Self::Error(message)
    }

    /// Generate an [ApiResponse] wrapping a [Response::Error]. This is intended for errors that
    /// are returned from fallible operations.
    pub fn error<E: Error>(error: E) -> Self {
        error!("{}", error);
        Self::Error(format!("{}", error))
    }
}
