use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use users::data::user::User;

use crate::{
    utils, utils::HtmxResponseBuilder, ServerFnError, EM_UID_SESSION_KEY, INTERNAL_SERVICE_ERROR,
    USERNAME_SESSION_KEY,
};

pub fn service() -> actix_web::Scope {
    web::scope("/login").route("", web::post().to(login_user))
}

#[derive(Deserialize, Serialize)]
pub struct Credentials {
    username: String,
    password: String,
}

pub async fn login_user(session: Session, credentials: web::Form<Credentials>) -> HttpResponse {
    let user = match login_user_api(credentials.0).await {
        Ok(inner) => inner,
        Err(_) => return HtmxResponseBuilder::new().static_body("Could not login user"),
    };
    if let Err(error) = session.insert(EM_UID_SESSION_KEY, *user.uid()) {
        log::error!("{error}");
        return utils::internal_server_error!();
    }
    if let Err(error) = session.insert(USERNAME_SESSION_KEY, user.username()) {
        log::error!("{error}");
        return utils::internal_server_error!();
    }
    HtmxResponseBuilder::location_home()
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
