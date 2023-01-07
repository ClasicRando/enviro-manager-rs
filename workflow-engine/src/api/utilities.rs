use rocket::form::{self, FromFormField, ValueField};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::serde::msgpack::MsgPack;
use serde::Serialize;

#[derive(Serialize)]
pub struct Response<T: Serialize> {
    code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T: Serialize> Response<T> {
    fn success(data: T) -> Self {
        Self {
            code: 200,
            message: None,
            data: Some(data),
        }
    }

    fn message(message: String) -> Self {
        Self {
            code: 200,
            message: Some(message),
            data: None,
        }
    }

    fn failure(message: String) -> Self {
        Self {
            code: 400,
            message: Some(message),
            data: None,
        }
    }

    fn error<E: std::error::Error>(error: E) -> Self {
        Self {
            code: 500,
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
    pub fn success(data: T, format: ApiResponse<FormatType>) -> Self {
        let response = Response::success(data);
        match format {
            ApiResponse::Json(_) => Self::Json(Json(response)),
            ApiResponse::MessagePack(_) => Self::MessagePack(MsgPack(response)),
        }
    }

    pub fn message(message: String, format: ApiResponse<FormatType>) -> Self {
        let response = Response::message(message);
        match format {
            ApiResponse::Json(_) => Self::Json(Json(response)),
            ApiResponse::MessagePack(_) => Self::MessagePack(MsgPack(response)),
        }
    }

    pub fn failure(message: String, format: ApiResponse<FormatType>) -> Self {
        let response = Response::failure(message);
        match format {
            ApiResponse::Json(_) => Self::Json(Json(response)),
            ApiResponse::MessagePack(_) => Self::MessagePack(MsgPack(response)),
        }
    }

    pub fn error<E: std::error::Error>(error: E, format: ApiResponse<FormatType>) -> Self {
        let response = Response::error(error);
        match format {
            ApiResponse::Json(_) => Self::Json(Json(response)),
            ApiResponse::MessagePack(_) => Self::MessagePack(MsgPack(response)),
        }
    }
}

#[derive(Serialize)]
pub struct FormatType;

#[rocket::async_trait]
impl<'r> FromFormField<'r> for ApiResponse<FormatType> {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match field.value {
            "json" => Ok(ApiResponse::Json(Json(Response::success(FormatType)))),
            _ => Ok(ApiResponse::MessagePack(MsgPack(Response::success(FormatType)))),
        }
    }

    fn default() -> Option<Self> {
        Some(ApiResponse::MessagePack(MsgPack(Response::success(FormatType))))
    }
}