use log::{error, warn};
use rocket::form::{self, FromFormField, ValueField};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::serde::msgpack::MsgPack;
use serde::Serialize;

#[derive(Serialize)]
pub struct Response<T: Serialize> {
    is_success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T: Serialize> Response<T> {
    fn success(data: T) -> Self {
        Self {
            is_success: true,
            message: None,
            data: Some(data),
        }
    }

    fn message(message: String) -> Self {
        Self {
            is_success: true,
            message: Some(message),
            data: None,
        }
    }

    fn failure(message: String) -> Self {
        Self {
            is_success: false,
            message: Some(message),
            data: None,
        }
    }

    fn error<E: std::error::Error>(error: E) -> Self {
        Self {
            is_success: false,
            message: Some(format!("Error: {}", error)),
            data: None,
        }
    }
}

#[derive(Responder)]
pub enum ApiResponse<T: Serialize> {
    #[response(status = 200, content_type = "json")]
    Json(Json<Response<T>>),
    #[response(status = 200, content_type = "msgpack")]
    MessagePack(MsgPack<Response<T>>),
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T, format: ApiFormatType) -> Self {
        let response = Response::success(data);
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }

    pub fn message(message: String, format: ApiFormatType) -> Self {
        let response = Response::message(message);
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }

    pub fn failure(message: String, format: ApiFormatType) -> Self {
        warn!("{}", message);
        let response = Response::failure(message);
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }

    pub fn error<E: std::error::Error>(error: E, format: ApiFormatType) -> Self {
        error!("{}", error);
        let response = Response::error(error);
        match format {
            ApiFormatType::Json => Self::Json(Json(response)),
            ApiFormatType::MessagePack => Self::MessagePack(MsgPack(response)),
        }
    }
}

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
