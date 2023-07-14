use actix_session::Session;
use actix_web::{web, HttpResponse};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use serde::Deserialize;
use users::service::users::{UpdateUserRequest, UpdateUserType, ValidateUserRequest};
use uuid::Uuid;

use crate::{
    components::{error_toast, RequestToast, UserEditSection, UserInfo},
    extract_session_uid, extract_session_username, utils, ServerFnError, USERNAME_SESSION_KEY,
};

pub fn service() -> actix_web::Scope {
    web::scope("/user")
        .route("/view-user", web::get().to(view_user))
        .service(
            web::resource("/edit-full-name")
                .route(web::get().to(edit_full_name))
                .route(web::post().to(update_full_name)),
        )
        .service(
            web::resource("/edit-username")
                .route(web::get().to(edit_username))
                .route(web::post().to(update_username)),
        )
    // .service(
    //     web::resource("/update-password")
    //         .route(web::get().to(update_password_form))
    //         .route(web::post().to(update_password)),
    // )
}

async fn edit_block(
    session: Session,
    edit_section: UserEditSection,
    toast_message: Option<String>,
) -> HttpResponse {
    let user = match utils::get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    let html = leptos::ssr::render_to_string(move |cx| {
        let toast = match toast_message {
            Some(message) => view! { cx, <RequestToast body=message/> }.into_view(cx),
            None => view! { cx, }.into_view(cx),
        };
        view! { cx,
            <UserInfo user=user edit_section=edit_section/>
            {toast}
        }
    });
    utils::html_chunk!(html)
}

async fn view_user(session: Session) -> HttpResponse {
    edit_block(session, UserEditSection::None, None).await
}

async fn edit_full_name(session: Session) -> HttpResponse {
    edit_block(session, UserEditSection::FullName, None).await
}

async fn edit_username(session: Session) -> HttpResponse {
    edit_block(session, UserEditSection::Username, None).await
}

#[derive(Deserialize)]
struct UpdateFullName {
    full_name: String,
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
        full_name,
        current_password,
    } = form.0;
    let update_user_type = UpdateUserType::FullName {
        new_name: full_name,
    };
    if let Err(error) = patch_user(uid, username, current_password, update_user_type).await {
        return error_toast!(error, "Could not update username");
    }
    utils::redirect_htmx!("/user")
}

#[derive(Deserialize)]
struct UpdateUsername {
    username: String,
    current_password: String,
}

async fn update_username(session: Session, form: web::Form<UpdateUsername>) -> HttpResponse {
    let Ok(uid) = extract_session_uid(&session) else {
        return utils::redirect_login!();
    };
    let Ok(current_username) = extract_session_username(&session) else {
        return utils::redirect_login!();
    };
    let UpdateUsername {
        username,
        current_password,
    } = form.0;
    let update_user_type = UpdateUserType::Username {
        new_username: username.clone(),
    };
    if let Err(error) = patch_user(uid, current_username, current_password, update_user_type).await
    {
        return error_toast!(error, "Could not update username");
    }
    if let Err(error) = session.insert(USERNAME_SESSION_KEY, &username) {
        log::error!("{error}");
        session.clear();
        return utils::redirect_login!();
    }
    utils::redirect_htmx!("/user")
}

// async fn update_password_form(session: Session) -> HttpResponse {
//     if extract_session_uid(&session).is_err() {
//         return utils::redirect_login!();
//     }
//     let html = leptos::ssr::render_to_string(|cx| {
//         view! { cx, <UpdatePassword /> }
//     });
//     utils::html_chunk!(html)
// }

// #[derive(Deserialize)]
// struct UpdatePassword {
//     new_password: String,
//     current_password: String,
// }

// async fn update_password(session: Session, form: web::Form<UpdatePassword>) -> HttpResponse {
//     let Ok(uid) = extract_session_uid(&session) else {
//         return utils::redirect_login!();
//     };
//     let Ok(username) = extract_session_username(&session) else {
//         return utils::redirect_login!();
//     };
//     let UpdatePassword {
//         new_password,
//         current_password,
//     } = form.0;
//     let update_user_type = UpdateUserType::ResetPassword { new_password };
//     if let Err(error) = patch_user(uid, username, current_password, update_user_type).await {
//         return error_toast!(error, "Could not update password");
//     }
//     toast!("Updated user's password!")
// }

async fn patch_user(
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
