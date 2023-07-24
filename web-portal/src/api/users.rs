use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use serde::Deserialize;
use users::{data::user::User, service::users::UpdateUserRequest};
use uuid::Uuid;

use crate::{
    components::{
        toast::{ADD_TOAST_SWAP, ADD_TOAST_TARGET},
        EditUser, Toast, UsersTable,
    },
    extract_session_uid, take_if, utils,
    utils::get_user,
    ServerFnError,
};

pub fn service() -> actix_web::Scope {
    web::scope("/users")
        .route("", web::get().to(all_users))
        .service(
            web::resource("/edit/{uid}")
                .route(web::post().to(edit_user_modal))
                .route(web::patch().to(edit_user)),
        )
}

async fn all_users(session: Session) -> HttpResponse {
    let Ok(uid) = extract_session_uid(&session) else {
        return utils::redirect_login_htmx!();
    };
    let users = match get_all_users(uid).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <UsersTable uid=uid users=users/> }
    });
    utils::html_chunk!(html)
}

pub async fn get_all_users(uid: Uuid) -> Result<Vec<User>, ServerFnError> {
    let users_response = utils::api_request(
        "http://127.0.0.1:8001/api/v1/users?f=msgpack",
        Method::GET,
        Some(uid),
        None::<()>,
    )
    .await?;
    let users = match users_response {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            return utils::server_fn_error!(message)
        }
    };
    Ok(users)
}

async fn edit_user_modal(session: Session, get_uid: web::Path<Uuid>) -> HttpResponse {
    let get_uid = get_uid.into_inner();
    let Ok(session_uid) = extract_session_uid(&session) else {
        return utils::redirect_login_htmx!();
    };

    let get_user = match get_user(session_uid, Some(get_uid)).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <EditUser user=get_user/> }
    });
    utils::html_chunk!(html)
}

#[derive(Deserialize)]
struct UserEditForm {
    username: String,
    full_name: String,
}

async fn edit_user(session: Session, form: web::Form<UserEditForm>) -> HttpResponse {
    let UserEditForm {
        username,
        full_name,
    } = form.into_inner();
    let Ok(session_uid) = extract_session_uid(&session) else {
        return utils::redirect_login_htmx!();
    };

    let update_request = UpdateUserRequest::new(
        session_uid,
        take_if(username, |s| !s.is_empty()),
        take_if(full_name, |s| !s.is_empty()),
    );

    if let Err(error) = update_user(session_uid, update_request).await {
        log::error!("{error}");
        let html = leptos::ssr::render_to_string(move |cx| {
            view! { cx, <Toast body="Could not update user"/> }
        });
        return HttpResponse::Ok()
            .insert_header(("HX-Retarget", ADD_TOAST_TARGET))
            .insert_header(("HX-Reswap", ADD_TOAST_SWAP))
            .body(html);
    }

    let users = match get_all_users(session_uid).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <UsersTable uid=session_uid users=users/> }
    });
    utils::html_chunk!(html)
}

async fn update_user(
    session_uid: Uuid,
    update_request: UpdateUserRequest,
) -> Result<(), ServerFnError> {
    let executors_response: ApiResponseBody<()> = utils::api_request(
        "http://127.0.0.1:8001/api/v1/users?f=msgpack",
        Method::PATCH,
        Some(session_uid),
        Some(update_request),
    )
    .await?;
    match executors_response {
        ApiResponseBody::Success(_) => {
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
