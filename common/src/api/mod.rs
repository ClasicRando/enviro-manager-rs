#[cfg(feature = "actix")]
pub mod actix;
#[cfg(feature = "actix")]
pub mod request;

use std::fmt::Debug;

use log::{error, warn};
use serde::{Deserialize, Serialize};

#[cfg(feature = "error")]
use crate::error::{EmError, EmResult};

/// Deserializable wrapper for allowing an API caller to send back content of an [ApiResponse].
/// This type should be used in a route handler to deserialize a url query with the template of
/// `?f={format}`.
#[derive(Deserialize, Default)]
pub struct QueryApiFormat {
    pub f: ApiContentFormat,
}

/// Format variants that an [ApiResponse] supports for serialization and deserialization of API
/// content
#[derive(Default, Deserialize, Clone, Copy)]
pub enum ApiContentFormat {
    #[serde(rename = "json")]
    Json,
    #[default]
    #[serde(rename = "msgpack")]
    MessagePack,
}

/// Generic response body for an [ApiResponse]. A response is either a success containing data, a
/// message to let the user know what happened or an error/failure message.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ApiResponseBody<T: Serialize> {
    Success(T),
    Message(String),
    Failure(String),
    Error(String),
}

/// API response object to enable serializing a `body` using the specified `format`. This type
/// can be used as a [Responder] for HTTP route handlers, always returning a 200 response unless
/// the serialization of the `body` fails.
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    #[serde(skip)]
    format: ApiContentFormat,
    #[serde(flatten)]
    body: ApiResponseBody<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Generate an [ApiResponse] wrapping a [ApiResponseBody::Success]`
    pub const fn success(data: T, format: ApiContentFormat) -> Self {
        Self {
            format,
            body: ApiResponseBody::Success(data),
        }
    }

    /// Generate an [ApiResponse] wrapping a [ApiResponseBody::Message]
    pub const fn message(message: String, format: ApiContentFormat) -> Self {
        Self {
            format,
            body: ApiResponseBody::Message(message),
        }
    }

    /// Generate an [ApiResponse] wrapping a [ApiResponseBody::Error]. This is intended for errors
    /// that are not runtime errors but rather user input issues.
    pub fn failure<S: Into<String>>(message: S, format: ApiContentFormat) -> Self {
        let failure_message = message.into();
        warn!("{}", failure_message);
        Self {
            format,
            body: ApiResponseBody::Failure(failure_message),
        }
    }

    /// Generate an [ApiResponse] for operations that return an [EmError]. Some [EmError] variants
    /// are downgraded to a [Failure][ApiResponseBody::Failure] if the `error` does not indicate an
    /// internal but rather bad user provided data or an error message the user could understand.
    #[cfg(feature = "error")]
    pub fn error<E>(error: E, format: ApiContentFormat) -> Self
    where
        E: Into<EmError>,
    {
        let error = error.into();
        error!("{}", error);
        match error {
            EmError::Generic(message) => Self::failure(message, format),
            EmError::InvalidUser
            | EmError::MissingRecord { .. }
            | EmError::InvalidRequest { .. }
            | EmError::InvalidPassword { .. }
            | EmError::MissingPrivilege { .. } => Self::failure(format!("{error}"), format),
            EmError::RmpDecode(_) => Self::failure("Could not decode the request object", format),
            _ => Self {
                format,
                body: ApiResponseBody::Error(
                    "Could not perform the required action due to an internal error".to_owned(),
                ),
            },
        }
    }
}

/// Validator for api requests that should have the request data verified
#[cfg(feature = "error")]
pub trait ApiRequestValidator {
    /// Type of the error message that is returned by the [validate][ApiRequestValidator::validate]
    /// method. Must be able to converted to a [String].
    type ErrorMessage: Into<String>;
    /// Type of request this validator is processing. Must implement debug to convert into an
    /// [EmError] type.
    type Request: Debug;
    /// Perform checks against the `request` to confirm it meets specified requirements. Returns an
    /// [Err] of a type that can be converted into a [String] if the request is not valid. Otherwise
    /// [Ok] is returned.
    /// # Errors
    /// This function will return an error if the `request` cannot be validated
    fn validate(request: &Self::Request) -> Result<(), Self::ErrorMessage>;
    /// Performs the implemented validation against the `request`, mapping the error (if any) into a
    /// specific validation [EmError]. If the validation succeeds, [Ok] is returned.
    /// # Errors
    /// This function will return an error if the `request` cannot be validated
    fn validate_request(request: &Self::Request) -> EmResult<()> {
        if let Err(error) = Self::validate(request) {
            return Err((request, error).into());
        }
        Ok(())
    }
}
