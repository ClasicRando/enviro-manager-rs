use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use serde::Deserialize;
use users::service::users::{UpdateUserRequest, UpdateUserType, ValidateUserRequest};
use uuid::Uuid;

use crate::{
    components::{UpdateFullName, UpdatePassword, UpdateUsername},
    extract_session_uid, extract_session_username, utils, ServerFnError, USERNAME_SESSION_KEY,
};

pub fn service() -> actix_web::Scope {
    web::scope("/user")
        .service(
            web::resource("/update-full-name")
                .route(web::get().to(update_full_name_form))
                .route(web::post().to(update_full_name)),
        )
        .service(
            web::resource("/update-username")
                .route(web::get().to(update_username_form))
                .route(web::post().to(update_username)),
        )
        .service(
            web::resource("/update-password")
                .route(web::get().to(update_password_form))
                .route(web::post().to(update_password)),
        )
}

async fn update_full_name_form(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login!();
    }
    let html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <UpdateFullName /> }
    });
    utils::html_chunk!(html)
}

#[derive(Deserialize)]
struct UpdateFullName {
    new_first_name: String,
    new_last_name: String,
    current_password: String,
}

async fn update_full_name(session: Session, form: web::Form<UpdateFullName>) -> HttpResponse {
    let Ok(uid) = extract_session_uid(&session) else {
        return utils::redirect_login!();
    };
    let Ok(username) = extract_session_username(&session) else {
        return utils::redirect_login!();
    };
    let UpdateFullName {
        new_first_name,
        new_last_name,
        current_password,
    } = form.0;
    let update_user_type = UpdateUserType::FullName {
        new_first_name,
        new_last_name,
    };
    if let Err(error) = patch_full_name(uid, username, current_password, update_user_type).await {
        return error.to_response();
    }
    utils::redirect_htmx!("/user")
}

async fn update_username_form(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login!();
    }
    let html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <UpdateUsername /> }
    });
    utils::html_chunk!(html)
}

#[derive(Deserialize)]
struct UpdateUsername {
    new_username: String,
    current_password: String,
}

async fn update_username(session: Session, form: web::Form<UpdateUsername>) -> HttpResponse {
    let Ok(uid) = extract_session_uid(&session) else {
        return utils::redirect_login!();
    };
    let Ok(username) = extract_session_username(&session) else {
        return utils::redirect_login!();
    };
    let UpdateUsername {
        new_username,
        current_password,
    } = form.0;
    let update_user_type = UpdateUserType::Username {
        new_username: new_username.clone(),
    };
    if let Err(error) = patch_full_name(uid, username, current_password, update_user_type).await {
        return error.to_response();
    }
    if let Err(error) = session.insert(USERNAME_SESSION_KEY, &new_username) {
        log::error!("{error}");
        session.clear();
        return utils::redirect_login!();
    }
    utils::redirect_htmx!("/user")
}

async fn update_password_form(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return utils::redirect_login!();
    }
    let html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <UpdatePassword /> }
    });
    utils::html_chunk!(html)
}

#[derive(Deserialize)]
struct UpdatePassword {
    new_password: String,
    current_password: String,
}

async fn update_password(session: Session, form: web::Form<UpdatePassword>) -> HttpResponse {
    let Ok(uid) = extract_session_uid(&session) else {
        return utils::redirect_login!();
    };
    let Ok(username) = extract_session_username(&session) else {
        return utils::redirect_login!();
    };
    let UpdatePassword {
        new_password,
        current_password,
    } = form.0;
    let update_user_type = UpdateUserType::ResetPassword { new_password };
    if let Err(error) = patch_full_name(uid, username, current_password, update_user_type).await {
        return error.to_response();
    }
    utils::redirect_htmx!("/user")
}

async fn patch_full_name(
    uid: Uuid,
    username: String,
    current_password: String,
    update_user_type: UpdateUserType,
) -> Result<(), ServerFnError> {
    let valid_user_request = ValidateUserRequest::new(username, current_password);
    let request = UpdateUserRequest::new(valid_user_request, update_user_type);

    let executors_response = utils::api_request(
        "http://127.0.0.1:8001/api/v1/users?f=msgpack",
        Method::PATCH,
        Some(uid),
        Some(request),
    )
    .await?;
    match executors_response {
        ApiResponseBody::Success(()) => {
            utils::server_fn_error!("Expected message, got data")
        }
        ApiResponseBody::Message(message) => {
            log::info!("{message}");
            Ok(())
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}
