use actix_session::Session;
use actix_web::{web, HttpResponse};
use leptos::view;
use users::data::{role::RoleName, user::User};
use workflow_engine::workflow_run::data::WorkflowRunId;

use crate::{
    api::{users::get_all_users, workflow_engine::get_workflow_run},
    components::{Index, Login, UserMissingRole, UsersPage, WorkflowEngine, WorkflowRunPage},
    extract_session_uid, utils, ServerFnError,
};

async fn login(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_ok() {
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
        view! { cx, <Index user=user/> }
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
        view! { cx, <WorkflowEngine user=user/> }
    });
    utils::html!(html)
}

async fn workflow_run(session: Session, workflow_run_id: web::Path<WorkflowRunId>) -> HttpResponse {
    let user = match utils::get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    let workflow_run = match get_workflow_run(workflow_run_id.into_inner()).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let mut html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <WorkflowRunPage user=user workflow_run=workflow_run/> }
    });
    utils::html!(html)
}

async fn users(session: Session) -> HttpResponse {
    let user = match utils::get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };

    let required_role = RoleName::Admin;
    if let Err(error) = user.check_role(required_role) {
        log::info!("{error}");
        return missing_role(user, required_role);
    }

    let users = match get_all_users(user.uid).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    let mut html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <UsersPage
                user=user
                users=users/>
        }
    });
    utils::html!(html)
}

fn missing_role(user: User, missing_role: RoleName) -> HttpResponse {
    let mut html = leptos::ssr::render_to_string(move |cx| {
        view! { cx, <UserMissingRole user=user missing_role=missing_role/> }
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
            .route(
                "/workflow-engine/workflow_run/{workflow_run_id}",
                web::get().to(workflow_run),
            )
            .route("/users", web::get().to(users))
    }
}
