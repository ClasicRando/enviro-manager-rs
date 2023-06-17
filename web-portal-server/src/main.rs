use std::fmt::Display;

use actix_cors::Cors;
use actix_web::{
    http::header,
    middleware::Logger,
    web::{get, post},
    App, HttpServer,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use common::{
    api::{request::ApiRequest, ApiContentFormat, ApiResponse, ApiResponseBody},
    error::EmResult,
};
use reqwest::{Client, IntoUrl, Method, Response};
use serde::{Deserialize, Serialize};
use users::data::user::User as ApiUser;
use web_portal_common::{Credentials, Session, User};

const SESSION_KEY: &str = "em_uid";
const USER_NOT_VALIDATED: &str = "Could not perform request. User not authenticated";

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("web-portal-server/api_server_log.yml", Default::default()).unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .route("/login", post().to(validate_user))
            .route("/user", get().to(fetch_user))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await?;
    Ok(())
}

/// Construct an [ApiResponse] using the raw `body` and `format`. Note, this method should not
/// be used if the `body` is [ApiResponseBody::Success] since the type and content of that
/// variant is lost during conversion.
fn from_body<B, T>(body: ApiResponseBody<B>, format: ApiContentFormat) -> ApiResponse<T>
where
    B: Serialize,
    T: Serialize,
{
    match body {
        ApiResponseBody::Success(_) => {
            ApiResponse::message("Body lost during transformation".to_owned(), format)
        }
        ApiResponseBody::Message(message) => ApiResponse::message(message, format),
        ApiResponseBody::Failure(message) => ApiResponse::failure(message, format),
        ApiResponseBody::Error(message) => ApiResponse::message(message, format),
    }
}

async fn send_request<U, D, T, R>(
    url: U,
    method: Method,
    auth: Option<D>,
    body: Option<T>,
) -> Result<Response, ApiResponse<R>>
where
    U: IntoUrl,
    D: Display,
    T: Serialize,
    R: Serialize,
{
    let client = Client::new();
    let mut builder = client.request(method, url);
    if let Some(auth) = auth {
        builder = builder.header("Authorization", format!("Bearer {auth}"))
    }
    if let Some(body) = body {
        let body = match rmp_serde::to_vec(&body) {
            Ok(inner) => inner,
            Err(error) => return Err(ApiResponse::error(error, ApiContentFormat::Json)),
        };
        builder = builder
            .body(body)
            .header("Content-Type", "application/msgpack")
    }
    match builder.send().await {
        Ok(response) => Ok(response),
        Err(error) => Err(ApiResponse::error(error, ApiContentFormat::Json)),
    }
}

async fn process_response<T, R>(response: Response) -> Result<T, ApiResponse<R>>
where
    T: Serialize + for<'de> Deserialize<'de>,
    R: Serialize,
{
    if !response.status().is_success() {
        return Err(ApiResponse::message(
            "Error while trying to login user".to_owned(),
            ApiContentFormat::Json,
        ));
    }
    let bytes = match response.bytes().await {
        Ok(inner) => inner,
        Err(error) => return Err(ApiResponse::error(error, ApiContentFormat::Json)),
    };
    let body = match rmp_serde::from_slice::<ApiResponseBody<T>>(&bytes) {
        Ok(inner) => inner,
        Err(error) => return Err(ApiResponse::error(error, ApiContentFormat::Json)),
    };
    match body {
        ApiResponseBody::Success(data) => Ok(data),
        _ => Err(from_body(body, ApiContentFormat::Json)),
    }
}

async fn validate_user(request: ApiRequest<Credentials>) -> ApiResponse<Session> {
    let login_result = send_request(
        "http://127.0.0.1:8001/api/v1/users/validate?f=msgpack",
        Method::POST,
        None::<String>,
        Some(request.into_inner()),
    )
    .await;
    let login_response = match login_result {
        Ok(response) => response,
        Err(api_response) => return api_response,
    };
    let user = match process_response::<ApiUser, _>(login_response).await {
        Ok(user) => user,
        Err(response) => {
            if let Ok(value) = serde_json::to_string(&response) {
                log::warn!("{}", value);
            }
            return response;
        }
    };
    ApiResponse::success(Session { token: *user.uid() }, ApiContentFormat::Json)
}

async fn fetch_user(bearer: BearerAuth) -> ApiResponse<User> {
    let user_result = send_request(
        "http://127.0.0.1:8001/api/v1/user?f=msgpack",
        Method::GET,
        Some(bearer.token()),
        None::<()>,
    )
    .await;
    let user_response = match user_result {
        Ok(response) => response,
        Err(api_response) => return api_response,
    };
    let user: ApiUser = match process_response(user_response).await {
        Ok(user) => user,
        Err(api_response) => return api_response,
    };
    return ApiResponse::success(
        User {
            full_name: user.full_name().to_owned(),
        },
        ApiContentFormat::Json,
    );
}
