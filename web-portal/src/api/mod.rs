pub mod login;
pub mod workflow_engine;

use actix_web::web;

pub fn service() -> actix_web::Scope {
    web::scope("/api")
        .service(login::service())
        .service(workflow_engine::service())
}
