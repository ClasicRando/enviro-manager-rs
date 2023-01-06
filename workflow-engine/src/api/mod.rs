use rocket::{routes, Build, Config, Rocket};

use crate::{
    executors_service, jobs_service, task_queue_service, tasks_service, workflow_runs_service,
    workflows_service, Result as WEResult,
};

pub async fn build_api() -> WEResult<Rocket<Build>> {
    let executors_service = executors_service().await?;
    let jobs_service = jobs_service().await?;
    let task_queue_service = task_queue_service().await?;
    let tasks_service = tasks_service().await?;
    let workflow_runs_service = workflow_runs_service().await?;
    let workflows_service = workflows_service().await?;
    let config = Config {
        port: 8000,
        ..Default::default()
    };
    let build = rocket::build()
        .manage(executors_service)
        .manage(jobs_service)
        .manage(task_queue_service)
        .manage(tasks_service)
        .manage(workflow_runs_service)
        .manage(workflows_service)
        .configure(config)
        .mount("/api/v1/", routes![]);
    Ok(build)
}
