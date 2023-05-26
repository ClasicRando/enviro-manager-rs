use std::fmt::Debug;

use actix_session::Session;
use actix_web::Responder;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{EmError, EmResult};

/// Generic response object as an API response. A response is either a success containing data, a
/// message to let the user know what happened or an error/failure message.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ApiResponse<T: Serialize> {
    Success(T),
    Message(String),
    Failure(String),
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
            .body(bytes.into_iter().collect::<actix_web::web::Bytes>())
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
    pub fn failure<S: AsRef<str>>(message: S) -> Self {
        warn!("{}", message.as_ref());
        Self::Failure(message.as_ref().to_owned())
    }

    /// Generate an [ApiResponse] wrapping a [Response::Error]. This is intended for errors that
    /// are returned from fallible operations.
    pub fn error(error: EmError) -> Self {
        error!("{}", error);
        match error {
            EmError::Generic(message) => Self::failure(message),
            EmError::InvalidUser
            | EmError::MissingRecord { .. }
            | EmError::InvalidRequest { .. }
            | EmError::InvalidPassword { .. }
            | EmError::MissingPrivilege { .. } => Self::failure(format!("{error}")),
            EmError::RmpDecode(_) => Self::failure("Could not decode the request object"),
            _ => Self::Error(
                "Could not perform the required action due to an internal error".to_owned(),
            ),
        }
    }
}

/// Validator for api requests that should have the request data verified
pub trait ApiRequestValidator {
    /// Type of request this validator is processing. Must implement debug to convert into an
    /// [EmError] type.
    type Request: Debug;
    /// Perform checks against the `request` to confirm it meets specified requirements. Returns an
    /// [Err] of [EmError][crate::error::EmError] if the request is not valid. Otherwise [Ok] is
    /// returned.
    /// # Errors
    ///
    fn validate(request: &Self::Request) -> EmResult<()>;
}

/// Validate that a `session` object contains the required data. Returns the users [Uuid] if the
/// session contains the key 'em_uid'. Otherwise, an [ApiResponse::Failure] is returned and should
/// be sent as the response to the request.
/// # Errors
/// This function will return an error if the `session` does not contain the key 'em_uid'.
pub fn validate_session<T: Serialize>(session: &Session) -> Result<Uuid, ApiResponse<T>> {
    let Some(uid) = session.get("em_uid").unwrap_or(None) else {
        return Err(ApiResponse::failure("Invalid or missing session ID"))
    };
    session.renew();
    Ok(uid)
}
