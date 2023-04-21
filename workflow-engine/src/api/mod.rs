mod executors;
mod jobs;
mod task_queue;
mod tasks;
mod utilities;
mod workflow_runs;
mod workflows;

use rocket::{routes, Build, Config, Rocket};

use crate::{
    create_executors_service, create_jobs_service, create_task_queue_service, create_tasks_service,
    create_workflow_runs_service, create_workflows_service, Result as WEResult, database::create_db_pool,
};

pub async fn build_api() -> WEResult<Rocket<Build>> {
    let pool = create_db_pool().await?;
    let executors_service = create_executors_service(&pool)?;
    let jobs_service = create_jobs_service(&pool)?;
    let task_queue_service = create_task_queue_service(&pool)?;
    let tasks_service = create_tasks_service(&pool)?;
    let workflow_runs_service = create_workflow_runs_service(&pool)?;
    let workflows_service = create_workflows_service(&pool)?;
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
        .mount(
            "/api/v1/",
            routes![
                tasks::tasks,
                tasks::task,
                tasks::create_task_json,
                tasks::create_task_msgpack,
                workflows::workflows,
                workflows::workflow,
                workflows::create_workflow_json,
                workflows::create_workflow_msgpack,
                workflows::deprecate_workflow_json,
                workflows::deprecate_workflow_msgpack,
                executors::active_executors,
                executors::shutdown_executor,
                executors::cancel_executor,
                workflow_runs::workflow_runs,
                workflow_runs::init_workflow_run,
                workflow_runs::cancel_workflow_run,
                workflow_runs::schedule_workflow_run,
                workflow_runs::restart_workflow_run,
                task_queue::task_queue_retry_json,
                task_queue::task_queue_retry_msgpack,
                task_queue::task_queue_complete_json,
                task_queue::task_queue_complete_msgpack,
                jobs::jobs,
                jobs::job,
                jobs::create_job_json,
                jobs::create_job_msgpack,
            ],
        );
    Ok(build)
}
