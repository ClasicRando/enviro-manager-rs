use actix_session::Session;
use actix_web::{http::header::ContentType, HttpResponse};
use askama::Template;

use crate::{utils, validate_session};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

pub async fn login(session: Session) -> HttpResponse {
    if validate_session(session).is_some() {
        return utils::redirect!("/");
    }
    let html = match LoginTemplate.render() {
        Ok(inner) => inner,
        Err(error) => return utils::internal_server_error!(error),
    };
    utils::html_response!(html)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub async fn index(session: Session) -> HttpResponse {
    let Some(_) = validate_session(session) else {
        return utils::redirect!("/login");
    };
    let html = match IndexTemplate.render() {
        Ok(inner) => inner,
        Err(error) => return utils::internal_server_error!(error),
    };
    utils::html_response!(html)
}
