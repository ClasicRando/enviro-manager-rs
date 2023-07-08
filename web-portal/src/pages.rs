use actix_session::Session;
use actix_web::HttpResponse;
use leptos::view;

use crate::{
    api::get_user,
    components::{Index, Login, WorkflowEngine},
    utils, validate_session, ServerFnError,
};

pub async fn login(session: Session) -> HttpResponse {
    if validate_session(session).is_ok() {
        return utils::redirect!("/");
    }
    let mut html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <Login /> }
    });
    utils::html!(html)
}

pub async fn index(session: Session) -> HttpResponse {
    let user = match get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    let mut html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <Index username=user.full_name().to_owned()/> }
    });
    utils::html!(html)
}

pub async fn workflow_engine(session: Session) -> HttpResponse {
    let user = match get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    let mut html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <WorkflowEngine username=user.full_name().to_owned()/> }
    });
    utils::html!(html)
}
