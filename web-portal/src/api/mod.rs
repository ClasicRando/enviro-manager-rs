pub mod login;
pub mod users;
pub mod workflow_engine;

use actix_web::web;
use serde::Deserialize;

#[derive(Deserialize)]
struct ModalIdQuery {
    id: String,
}

pub fn service() -> actix_web::Scope {
    web::scope("/api")
        .service(login::service())
        .service(workflow_engine::service())
        .service(users::service())
}
