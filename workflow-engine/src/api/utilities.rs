use log::{error, warn};
use rocket::form::{self, FromFormField, ValueField};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::serde::msgpack::MsgPack;
use serde::Serialize;

/// Generic response object within an API response. A response is either a success containing data,
/// a message to let the user know what happened or an error/failure message.
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Response<T: Serialize> {
    Success(T),
    Message(String),
    Error(String),
}

/// [Responder] object for the workflow engine API. The underlining [Response] object can be
/// serialized as:
/// 
/// - JSON
/// - MessagePack
/// 
/// Although there is no deep link between the [ApiResponse] enum and the [ApiFormatType] enum the
/// two types should be kept with the same options to ensure expected behaviour.
#[derive(Responder)]
pub enum ApiResponse<T: Serialize> {
    #[response(status = 200, content_type = "json")]
    Json(Json<Response<T>>),
    #[response(status = 200, content_type = "msgpack")]
    MessagePack(MsgPack<Response<T>>),
}

impl<T: Serialize> ApiResponse<T> {
    /// Generate a [Response::Success] responder of the specified `format`
    pub fn success(data: T, format: ApiFormatType) -> Self {
        let response = Response::Success(data);
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }

    /// Generate a [Response::Message] responder of the specified `format`
    pub fn message(message: String, format: ApiFormatType) -> Self {
        let response = Response::Message(message);
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }

    /// Generate a [Response::Error] responder of the specified `format`. This is intended for
    /// errors that were not runtime errors but rather user input issues.
    pub fn failure(message: String, format: ApiFormatType) -> Self {
        warn!("{}", message);
        let response = Response::Error(message);
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }

    /// Generate a [Response::Error] responder of the specified `format`. This is intended for
    /// errors that are returned from fallible operations.
    pub fn error<E: std::error::Error>(error: E, format: ApiFormatType) -> Self {
        error!("{}", error);
        let response = Response::Error(format!("{}", error));
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }
}

/// API response formatting types. The allowed serialization methods are:
/// 
/// - JSON
/// - MessagePack
/// 
/// Although there is no deep link between the [ApiResponse] enum and the [ApiFormatType] enum the
/// two types should be kept with the same options to ensure expected behaviour.
pub enum ApiFormatType {
    Json,
    MessagePack,
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for ApiFormatType {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match field.value {
            "json" => Ok(ApiFormatType::Json),
            _ => Ok(ApiFormatType::MessagePack),
        }
    }

    fn default() -> Option<Self> {
        Some(ApiFormatType::MessagePack)
    }
}
