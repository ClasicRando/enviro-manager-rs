use std::{
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};

use actix_web::{
    dev::Payload, error::PayloadError, http::header, web::BytesMut, FromRequest, HttpMessage,
    HttpRequest,
};
use actix_web_lab::__reexports::futures_util::Stream;
use log::error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

use super::ApiContentFormat;
use crate::error::{EmError, EmResult};

/// Generic API request containing the extracted body of a request object. This type is
/// constrained to requests that have a `Content-Type` header that matches the labels in the
/// [ApiContentFormat] enum. Otherwise, you will encounter a error response before entering the
/// route handler body.
///
/// If your request does match the expected `Content-Type` options, this type can be used in a
/// route handler to extract the request body into the desired type `T`.
#[derive(Serialize, Deserialize)]
pub struct ApiRequest<T>(T);

impl<T> ApiRequest<T> {
    /// Extract the inner contents of the request after deserializing from a request
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum ApiRequestPayloadError {
    /// Payload size is bigger than allowed & content length header set. (default: 2MB)
    #[error("API request payload ({length} bytes) is larger than allowed (limit: {limit} bytes).")]
    OverflowKnownLength { length: usize, limit: usize },
    /// Payload size is bigger than allowed but no content length header set. (default: 2MB)
    #[error("JSON payload has exceeded limit ({limit} bytes).")]
    Overflow { limit: usize },
    /// Content type error
    #[error("Content type error. No content type included in request")]
    NoContentType,
    /// Content type error
    #[error("Content type error. Expected json or msgpack but got {:?}", _0.subtype())]
    ContentType(mime::Mime),
    /// Deserialize error
    #[error("Json deserialize error: {0}")]
    JsonDeserialize(serde_json::Error),
    /// Serialize error
    #[error("Json serialize error: {0}")]
    JsonSerialize(serde_json::Error),
    /// Deserialize error
    #[error("Msgpack deserialize error: {0}")]
    MsgpackDeserialize(rmp_serde::decode::Error),
    /// Serialize error
    #[error("Msgpack serialize error: {0}")]
    MsgpackSerialize(rmp_serde::encode::Error),
    /// Payload error
    #[error("Error that occur during reading payload: {0}")]
    Payload(#[from] PayloadError),
}

impl<T: DeserializeOwned> FromRequest for ApiRequest<T> {
    type Error = EmError;
    type Future = ApiRequestExtractFut<T>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let config = ApiRequestConfig::from_req(req);

        let limit = config.limit;
        let err_handler = config.err_handler.clone();

        ApiRequestExtractFut {
            req: Some(req.clone()),
            fut: ApiRequestBody::new(req, payload).limit(limit),
            err_handler,
        }
    }
}

/// Type alias for a function that processes an [ApiRequestPayloadError] and a borrowed
/// [HttpRequest] to return an [EmError]. This is meant to handle an error parsing the payload of
/// an API request into an [ApiRequest].
type ApiRequestErrorHandler =
    Option<Arc<dyn Fn(ApiRequestPayloadError, &HttpRequest) -> EmError + Send + Sync>>;

/// Future type that processes an [HttpRequest] and returns the result of another future,
/// [ApiRequestBody]. This nested future processes the request body into the generic type `T` to
/// which this future wraps an successful parsing result into an [ApiRequest] result.
pub struct ApiRequestExtractFut<T> {
    req: Option<HttpRequest>,
    fut: ApiRequestBody<T>,
    err_handler: ApiRequestErrorHandler,
}

#[allow(clippy::unwrap_used)]
impl<T: DeserializeOwned> Future for ApiRequestExtractFut<T> {
    type Output = EmResult<ApiRequest<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let res = ready!(Pin::new(&mut this.fut).poll(cx));

        let res = match res {
            Err(err) => {
                let req = this.req.take().unwrap();
                log::debug!(
                    "Failed to deserialize Json from payload. Request path: {}",
                    req.path()
                );

                if let Some(err_handler) = this.err_handler.as_ref() {
                    Err((*err_handler)(err, &req))
                } else {
                    Err(err.into())
                }
            }
            Ok(data) => Ok(ApiRequest(data)),
        };

        Poll::Ready(res)
    }
}

/// `ApiRequest` extractor configuration.
#[derive(Clone)]
pub struct ApiRequestConfig {
    limit: usize,
    err_handler: ApiRequestErrorHandler,
}

impl ApiRequestConfig {
    /// Set maximum accepted payload size. By default this limit is 2MB.
    pub const fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set custom error handler.
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(ApiRequestPayloadError, &HttpRequest) -> EmError + Send + Sync + 'static,
    {
        self.err_handler = Some(Arc::new(f));
        self
    }

    /// Extract payload config from app data. Check both `T` and `Data<T>`, in that order, and fall
    /// back to the default payload config.
    fn from_req(req: &HttpRequest) -> &Self {
        req.app_data::<Self>()
            .or_else(|| {
                req.app_data::<actix_web::web::Data<Self>>()
                    .map(|d| d.as_ref())
            })
            .unwrap_or(&DEFAULT_CONFIG)
    }
}

const DEFAULT_LIMIT: usize = 2_097_152; // 2 mb

/// Allow shared refs used as default.
const DEFAULT_CONFIG: ApiRequestConfig = ApiRequestConfig {
    limit: DEFAULT_LIMIT,
    err_handler: None,
};

impl Default for ApiRequestConfig {
    fn default() -> Self {
        DEFAULT_CONFIG
    }
}

/// Future that resolves to some `T` when parsed from a valid API request payload.
///
/// Can deserialize any type `T` that implements [`Deserialize`][serde::Deserialize].
///
/// Returns error if:
/// - `Content-Type` does not match any content within [ApiContentFormat]
/// - `Content-Length` is greater than [limit](ApiRequestBody::limit()).
/// - The payload, when consumed, does not match the specified `Content-Type`
pub enum ApiRequestBody<T> {
    Error(Option<ApiRequestPayloadError>),
    Body {
        limit: usize,
        content_type: ApiContentFormat,
        /// Length as reported by `Content-Length` header, if present.
        length: Option<usize>,
        payload: Payload,
        buf: BytesMut,
        _res: PhantomData<T>,
    },
}

impl<T> Unpin for ApiRequestBody<T> {}

impl<T: DeserializeOwned> ApiRequestBody<T> {
    /// Create a new future to decode a JSON request payload.
    #[allow(clippy::borrow_interior_mutable_const)]
    pub fn new(req: &HttpRequest, payload: &mut Payload) -> Self {
        // check content-type
        let Ok(Some(mime)) = req.mime_type() else {
            return Self::Error(Some(ApiRequestPayloadError::NoContentType));
        };
        let Some(api_content_type) = ApiContentFormat::from_mime(&mime) else {
            return Self::Error(Some(ApiRequestPayloadError::ContentType(mime)));
        };

        let length = req
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|l| l.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok());

        // Notice the content-length is not checked against limit of json config here.
        // As the internal usage always call JsonBody::limit after JsonBody::new.
        // And limit check to return an error variant of JsonBody happens there.

        Self::Body {
            limit: DEFAULT_LIMIT,
            content_type: api_content_type,
            length,
            payload: payload.take(),
            buf: BytesMut::with_capacity(8192),
            _res: PhantomData,
        }
    }

    /// Set maximum accepted payload size. The default limit is 2MB.
    pub fn limit(self, limit: usize) -> Self {
        match self {
            Self::Body {
                length,
                content_type,
                payload,
                buf,
                ..
            } => {
                if let Some(len) = length {
                    if len > limit {
                        return Self::Error(Some(ApiRequestPayloadError::OverflowKnownLength {
                            length: len,
                            limit,
                        }));
                    }
                }

                Self::Body {
                    limit,
                    content_type,
                    length,
                    payload,
                    buf,
                    _res: PhantomData,
                }
            }
            Self::Error(e) => Self::Error(e),
        }
    }
}

#[allow(clippy::unwrap_used)]
impl<T: DeserializeOwned> Future for ApiRequestBody<T> {
    type Output = Result<T, ApiRequestPayloadError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match this {
            Self::Body {
                limit,
                content_type,
                buf,
                payload,
                ..
            } => loop {
                let res = ready!(Pin::new(&mut *payload).poll_next(cx));
                match res {
                    Some(chunk) => {
                        let chunk = chunk?;
                        let buf_len = buf.len() + chunk.len();
                        if buf_len > *limit {
                            return Poll::Ready(Err(ApiRequestPayloadError::Overflow {
                                limit: *limit,
                            }));
                        } else {
                            buf.extend_from_slice(&chunk);
                        }
                    }
                    None => {
                        let result = match content_type {
                            ApiContentFormat::Json => serde_json::from_slice::<T>(buf)
                                .map_err(ApiRequestPayloadError::JsonDeserialize)?,
                            ApiContentFormat::MessagePack => rmp_serde::from_slice::<T>(buf)
                                .map_err(ApiRequestPayloadError::MsgpackDeserialize)?,
                        };
                        return Poll::Ready(Ok(result));
                    }
                }
            },
            Self::Error(e) => Poll::Ready(Err(e.take().unwrap())),
        }
    }
}
