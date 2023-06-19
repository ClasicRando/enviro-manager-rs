pub mod user;

use std::fmt::Display;

use common::api::ApiResponseBody;
use leptos::ServerFnError;
use reqwest::{Client, IntoUrl, Method, Response};
use serde::{Deserialize, Serialize};

mod utils {
    macro_rules! server_fn_error {
        ($f:literal, $($item:ident)+) => {
            Err(ServerFnError::ServerError(format!($f, $($item)+)))
        };
    }
    pub(crate) use server_fn_error;
}

async fn send_request<U, D, T>(
    url: U,
    method: Method,
    auth: Option<D>,
    body: Option<T>,
) -> Result<Response, ServerFnError>
where
    U: IntoUrl,
    D: Display,
    T: Serialize,
{
    let client = Client::new();
    let mut builder = client.request(method, url);
    if let Some(auth) = auth {
        builder = builder.header("Authorization", format!("Bearer {auth}"))
    }
    if let Some(body) = body {
        let body = match rmp_serde::to_vec(&body) {
            Ok(inner) => inner,
            Err(error) => {
                return Err(ServerFnError::Serialization(format!(
                    "Api request body cannot be serialized. {error}"
                )))
            }
        };
        builder = builder
            .body(body)
            .header("Content-Type", "application/msgpack")
    }
    match builder.send().await {
        Ok(response) => Ok(response),
        Err(error) => Err(ServerFnError::Request(format!(
            "Error performing api request. {error}"
        ))),
    }
}

async fn process_response<T>(response: Response) -> Result<ApiResponseBody<T>, ServerFnError>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    if !response.status().is_success() {
        return Err(ServerFnError::Request(format!(
            "Error performing api request, {}",
            response.status()
        )));
    }
    let bytes = match response.bytes().await {
        Ok(inner) => inner,
        Err(error) => {
            return Err(ServerFnError::ServerError(format!(
                "Api response body cannot be processed. {error}"
            )))
        }
    };
    match rmp_serde::from_slice::<ApiResponseBody<T>>(&bytes) {
        Ok(inner) => Ok(inner),
        Err(error) => Err(ServerFnError::Deserialization(format!(
            "Api response body cannot be deserialized. {error}"
        ))),
    }
}
