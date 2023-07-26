pub mod login;
pub mod users;
pub mod workflow_engine;

use actix_web::{web, HttpResponse, HttpResponseBuilder};
use serde::Deserialize;
use serde_json::json;

use crate::components::modal::MODAL_ERROR_MESSAGE_ID;

#[derive(Deserialize)]
struct ModalIdQuery {
    id: String,
}

struct HtmxResponseBuilder {
    response: HttpResponseBuilder,
    triggers: Option<Vec<(&'static str, serde_json::Value)>>,
}

impl HtmxResponseBuilder {
    fn new() -> Self {
        let mut response = HttpResponse::Ok();
        response.content_type(actix_web::http::header::ContentType::html());
        Self {
            response,
            triggers: None,
        }
    }

    fn add_close_modal_event<S>(&mut self, modal_id: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.add_trigger_event("closeModal", json!({"id": modal_id.as_ref()}))
    }

    fn add_create_toast_event<S>(&mut self, message: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.add_trigger_event("createToast", json!({"message": message.as_ref()}))
    }

    fn add_trigger_event(&mut self, event: &'static str, data: serde_json::Value) -> &mut Self {
        match self.triggers.as_mut() {
            Some(triggers) => triggers.push((event, data)),
            None => self.triggers = Some(vec![(event, data)]),
        };
        self
    }

    fn redirect<S>(&mut self, location: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.response
            .insert_header(("HX-Redirect", location.as_ref()));
        self
    }

    fn target<S>(&mut self, target: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.response
            .insert_header(("HX-Retarget", target.as_ref()));
        self
    }

    fn swap<S>(&mut self, swap: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.response.insert_header(("HX-Swap", swap.as_ref()));
        self
    }

    fn finish_triggers(&mut self) -> &mut Self {
        let triggers_option = self.triggers.take();
        match triggers_option {
            Some(triggers) if !triggers.is_empty() => {
                let data = triggers
                    .into_iter()
                    .map(|(key, obj)| json!({key: obj}).to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                self.response
                    .insert_header(("HX-Trigger", format!("{{{}}}", data)));
            }
            _ => {}
        }
        self
    }

    fn modal_error_message<S>(message: S) -> HttpResponse
    where
        S: Into<String>,
    {
        Self::new()
            .target(MODAL_ERROR_MESSAGE_ID)
            .swap("innerHTML")
            .raw_body(message.into())
    }

    fn raw_body(&mut self, html: String) -> HttpResponse {
        self.finish_triggers();
        self.response.body(html)
    }

    fn html_chunk<F, IV>(&mut self, html: F) -> HttpResponse
    where
        F: FnOnce(leptos::Scope) -> IV + 'static,
        IV: leptos::IntoView,
    {
        self.finish_triggers();
        let html = leptos::ssr::render_to_string(html);
        self.response.body(html)
    }

    fn page<F, IV>(&mut self, html: F) -> HttpResponse
    where
        F: FnOnce(leptos::Scope) -> IV + 'static,
        IV: leptos::IntoView,
    {
        self.finish_triggers();
        let mut html = leptos::ssr::render_to_string(html);
        html.insert_str(0, "<!DOCTYPE html>");
        self.response.body(html)
    }
}

pub fn service() -> actix_web::Scope {
    web::scope("/api")
        .service(login::service())
        .service(workflow_engine::service())
        .service(users::service())
}
