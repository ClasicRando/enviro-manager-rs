use actix_session::Session;
use actix_web::{web, HttpResponse};
use leptos::view;

use crate::{
    components::{Index, Login, WorkflowEngine},
    utils, validate_session, ServerFnError,
};

async fn login(session: Session) -> HttpResponse {
    if validate_session(&session).is_ok() {
        return utils::redirect!("/");
    }
    let mut html = leptos::ssr::render_to_string(|cx| {
        view! { cx, <Login /> }
    });
    utils::html!(html)
}

async fn index(session: Session) -> HttpResponse {
    let user = match utils::get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    let mut html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <Index username=user.full_name().to_owned()/> }
    });
    utils::html!(html)
}

async fn workflow_engine(session: Session) -> HttpResponse {
    let user = match utils::get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    let mut html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <WorkflowEngine username=user.full_name().to_owned()/> }
    });
    utils::html!(html)
}

async fn logout_user(session: Option<Session>) -> HttpResponse {
    if let Some(session) = session {
        session.clear()
    }
    utils::redirect!("/login")
}

async fn redirect_home() -> HttpResponse {
    utils::redirect_home!()
}

pub trait Pages {
    fn add_pages(self) -> Self;
}

impl<T> Pages for actix_web::App<T>
where
    T: actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Error = actix_web::error::Error,
        InitError = (),
    >,
{
    fn add_pages(self) -> Self {
        self.route("/", web::get().to(index))
            .route("/index", web::get().to(redirect_home))
            .route("/login", web::get().to(login))
            .route("/workflow-engine", web::get().to(workflow_engine))
            .route("/logout", web::get().to(logout_user))
    }
}
