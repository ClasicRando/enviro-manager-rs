use actix_session::Session;
use actix_web::HttpResponse;
use askama::Template;
use askama_actix::TemplateToResponse;

use crate::{api::get_user, utils, validate_session, ServerFnError};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    title: &'static str,
    user_name: &'static str,
}

impl LoginTemplate {
    fn to_response() -> HttpResponse {
        Self {
            title: "Login",
            user_name: "",
        }
        .to_response()
    }
}

pub async fn login(session: Session) -> HttpResponse {
    if validate_session(session).is_ok() {
        return utils::redirect!("/");
    }
    LoginTemplate::to_response()
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'n> {
    title: &'static str,
    user_name: &'n str,
}

impl<'n> IndexTemplate<'n> {
    fn new(user_name: &'n str) -> Self {
        Self {
            title: "Home",
            user_name,
        }
    }
}

pub async fn index(session: Session) -> HttpResponse {
    let user = match get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    IndexTemplate::new(user.full_name()).to_response()
}

#[derive(Template)]
#[template(path = "workflow-engine.html")]
struct WorkflowEngineTemplate<'n> {
    title: &'static str,
    user_name: &'n str,
}

impl<'n> WorkflowEngineTemplate<'n> {
    fn new(user_name: &'n str) -> Self {
        Self {
            title: "Workflow Engine",
            user_name,
        }
    }
}

pub async fn workflow_engine(session: Session) -> HttpResponse {
    let user = match get_user(session).await {
        Ok(inner) => inner,
        Err(ServerFnError::InvalidUser) => return utils::redirect_login!(),
        Err(error) => return error.to_response(),
    };
    WorkflowEngineTemplate::new(user.full_name()).to_response()
}
