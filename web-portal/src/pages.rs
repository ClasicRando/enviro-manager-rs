use actix_session::Session;
use actix_web::{web, HttpResponse};
use leptos::*;
use users::data::{role::RoleName, user::User};
use workflow_engine::workflow_run::data::WorkflowRunId;

use crate::{
    api::{users::get_all_users, workflow_engine::workflow_run::get_workflow_run},
    components::{
        base::BasePage,
        error::UserMissingRole,
        login::LoginForm,
        users::UsersTable,
        workflow_engine::{
            main_page::default_workflow_engine_tab_url, workflow_run_page::WorkflowRunDisplay,
        },
    },
    extract_session_uid,
    utils::{self, html_page, HtmxResponseBuilder, HOME_LOCATION},
    ServerFnError,
};

async fn login(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_ok() {
        return utils::redirect_home!();
    }
    html_page(|cx| {
        view! { cx,
            <BasePage title="Index">
                <LoginForm />
            </BasePage>
        }
    })
}

async fn index(session: Session) -> HttpResponse {
    let user = match utils::get_user_session(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    html_page(|cx| {
        view! { cx,
            <BasePage title="Index" user=user>
            </BasePage>
        }
    })
}

async fn workflow_engine(session: Session) -> HttpResponse {
    let user = match utils::get_user_session(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    html_page(|cx| {
        view! { cx,
            <BasePage title="Index" user=user>
                <div id="tabs" hx-get={default_workflow_engine_tab_url()} hx-trigger="load"
                    hx-target="#tabs" hx-swap="innerHTML"></div>
            </BasePage>
        }
    })
}

async fn workflow_run(session: Session, workflow_run_id: web::Path<WorkflowRunId>) -> HttpResponse {
    let user = match utils::get_user_session(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    let workflow_run = match get_workflow_run(workflow_run_id.into_inner()).await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };
    html_page(|cx| {
        view! { cx,
            <BasePage title="Workflow Run" user=user>
                <WorkflowRunDisplay workflow_run=workflow_run/>
            </BasePage>
        }
    })
}

async fn users(session: Session) -> HttpResponse {
    let user = match utils::get_user_session(session).await {
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
    let uid = user.uid;
    html_page(move |cx| {
        view! { cx,
            <BasePage title="Users" user=user>
                <UsersTable uid=uid users=users/>
            </BasePage>
        }
    })
}

fn missing_role(user: User, missing_role: RoleName) -> HttpResponse {
    html_page(move |cx| {
        view! { cx, <UserMissingRole user=user missing_role=missing_role/> }
    })
}

async fn logout_user(session: Option<Session>) -> HttpResponse {
    if let Some(session) = session {
        session.clear()
    }
    HtmxResponseBuilder::location_login()
}

async fn redirect_home() -> HttpResponse {
    HttpResponse::Found()
        .insert_header(("location", HOME_LOCATION))
        .finish()
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
            .service(
                web::scope("/workflow-engine")
                    .route("", web::get().to(workflow_engine))
                    .route(
                        "/workflow-run/{workflow_run_id}",
                        web::get().to(workflow_run),
                    ),
            )
            .route("/logout", web::get().to(logout_user))
            .route("/users", web::get().to(users))
    }
}
