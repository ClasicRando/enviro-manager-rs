use actix_web::Responder;
use log::error;
use serde::Serialize;

use super::{ApiContentFormat, ApiResponse};
use crate::error::EmError;

impl ApiContentFormat {
    pub fn from_mime(value: &mime::Mime) -> Option<Self> {
        if value.subtype() == mime::JSON || value.suffix() == Some(mime::JSON) {
            return Some(Self::Json);
        }
        if value.subtype() == mime::MSGPACK || value.suffix() == Some(mime::MSGPACK) {
            return Some(Self::MessagePack);
        }
        None
    }
}

impl<T> Responder for ApiResponse<T>
where
    T: Serialize + 'static,
{
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let bytes_result: Result<Vec<u8>, EmError> = match self.format {
            ApiContentFormat::Json => serde_json::to_vec(&self.body).map_err(|e| e.into()),
            ApiContentFormat::MessagePack => rmp_serde::to_vec(&self.body).map_err(|e| e.into()),
        };
        let bytes = match bytes_result {
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
            .content_type(actix_web::http::header::ContentType(match self.format {
                ApiContentFormat::Json => mime::APPLICATION_JSON,
                ApiContentFormat::MessagePack => mime::APPLICATION_MSGPACK,
            }))
            .body(bytes.into_iter().collect::<actix_web::web::Bytes>())
    }
}
