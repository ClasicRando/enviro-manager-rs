use actix_web::{http::header::ContentType, HttpResponse};
use askama::Template;

macro_rules! internal_server_error {
    ($error:ident) => {
        HttpResponse::InternalServerError().body(format!("{}", $error))
    };
}

macro_rules! html_response {
    ($html:ident) => {
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body($html)
    };
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

pub async fn login() -> HttpResponse {
    let html = match LoginTemplate.render() {
        Ok(inner) => inner,
        Err(error) => return internal_server_error!(error),
    };
    html_response!(html)
}
