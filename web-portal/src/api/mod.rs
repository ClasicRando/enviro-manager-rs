mod workflow_engine;

use actix_session::Session;
use actix_web::{
    web::{self, Form},
    HttpResponse,
};
use common::api::ApiResponseBody;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use users::data::user::User;

use crate::{utils, ServerFnError, INTERNAL_SERVICE_ERROR, SESSION_KEY};

pub fn service() -> actix_web::Scope {
    web::scope("/api")
        .route("/login", web::post().to(login_user))
        .service(workflow_engine::service())
}

#[derive(Deserialize, Serialize)]
struct Credentials {
    username: String,
    password: String,
}

async fn login_user(session: Session, credentials: Form<Credentials>) -> HttpResponse {
    let user = match login_user_api(credentials.0).await {
        Ok(inner) => inner,
        Err(_) => return utils::html_chunk!("Could not login user"),
    };
    if let Err(error) = session.insert(SESSION_KEY, *user.uid()) {
        log::error!("{error}");
        return utils::internal_server_error!();
    }
    utils::redirect_home_htmx!()
}

async fn login_user_api(credentials: Credentials) -> Result<User, ServerFnError> {
    let user_response = utils::api_request(
        "http://127.0.0.1:8001/api/v1/users/validate?f=msgpack",
        Method::POST,
        None::<String>,
        Some(credentials),
    )
    .await?;
    let user = match user_response {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            log::error!("Expected data, got message. {message}");
            return utils::server_fn_error!("Expected data, got message. {}", message);
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            log::error!("{message}");
            return utils::server_fn_static_error!(INTERNAL_SERVICE_ERROR);
        }
    };
    Ok(user)
}
