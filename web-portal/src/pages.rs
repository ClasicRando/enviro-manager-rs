use actix_session::Session;
use actix_web::HttpResponse;
use askama::Template;
use askama_actix::TemplateToResponse;

use crate::{api::get_user, utils, validate_session};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

impl LoginTemplate {
    fn to_response() -> HttpResponse {
        Self {}.to_response()
    }
}

pub async fn login(session: Session) -> HttpResponse {
    if validate_session(session).is_some() {
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
        IndexTemplate {
            title: "Home",
            user_name,
        }
    }
}

pub async fn index(session: Session) -> HttpResponse {
    let Some(user) = validate_session(session) else {
        return utils::redirect!("/login");
    };
    let user = match get_user(user).await {
        Ok(inner) => inner,
        Err(error) => {
            log::error!("{error}");
            return IndexTemplate::new("Test").to_response();
            // return utils::internal_server_error!();
        }
    };
    IndexTemplate::new(user.full_name()).to_response()
}
