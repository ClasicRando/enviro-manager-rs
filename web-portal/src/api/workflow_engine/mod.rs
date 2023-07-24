mod executors;
pub mod workflow_run;
mod workflow_runs;
mod workflows;

use actix_web::web;

pub fn service() -> actix_web::Scope {
    web::scope("/workflow-engine")
        .service(executors::service())
        .service(workflow_run::service())
        .service(workflow_runs::service())
}
