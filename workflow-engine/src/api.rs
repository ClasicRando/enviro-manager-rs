use std::net::ToSocketAddrs;

use actix_web::{
    web::{get, patch, post, Data},
    App, HttpServer,
};
use common::{database::Database, error::EmResult};

use crate::{
    executor::{api as executors_api, service::ExecutorService},
    job::{api as jobs_api, service::JobService},
    workflow::{
        api as workflows_api,
        service::{TaskService, WorkflowsService},
    },
    workflow_run::{
        api as workflow_runs_api,
        service::{TaskQueueService, WorkflowRunsService},
    },
};

/// Temp
/// # Errors
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub async fn spawn_api_server<A, D, E, J, Q, R, T, W>(
    executor_service: E,
    workflow_run_service: R,
    task_queue_service: Q,
    task_service: T,
    workflow_service: W,
    job_service: J,
    address: A,
) -> EmResult<()>
where
    A: ToSocketAddrs,
    D: Database,
    E: ExecutorService<Database = D> + Send + Sync + 'static,
    J: JobService<Database = D, WorkflowRunService = R> + Send + Sync + 'static,
    Q: TaskQueueService<Database = D, WorkflowRunService = R> + Send + Sync + 'static,
    R: WorkflowRunsService<Database = D> + Send + Sync + 'static,
    T: TaskService<Database = D> + Send + Sync + 'static,
    W: WorkflowsService<Database = D> + Send + Sync + 'static,
{
    let executors_service_data = Data::new(executor_service);
    let workflow_runs_service_data = Data::new(workflow_run_service);
    let task_queue_service_data = Data::new(task_queue_service);
    let tasks_service_data = Data::new(task_service);
    let workflows_service_data = Data::new(workflow_service);
    let jobs_service_data = Data::new(job_service);
    HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .app_data(executors_service_data.clone())
                .app_data(jobs_service_data.clone())
                .app_data(task_queue_service_data.clone())
                .app_data(tasks_service_data.clone())
                .app_data(workflow_runs_service_data.clone())
                .app_data(workflows_service_data.clone())
                .route("/executors", get().to(executors_api::active_executors::<E>))
                .route(
                    "/executors/shutdown/{executor_id}",
                    get().to(executors_api::shutdown_executor::<E>),
                )
                .route(
                    "/executors/cancel/{executor_id}",
                    get().to(executors_api::cancel_executor::<E>),
                )
                .route("/jobs", get().to(jobs_api::jobs::<J>))
                .route("/jobs/{job_id}", get().to(jobs_api::job::<J>))
                .route("/jobs", post().to(jobs_api::create_job::<J>))
                .route(
                    "/task-queue/retry",
                    post().to(workflow_runs_api::task_queue_retry::<Q>),
                )
                .route(
                    "/task-queue/complete",
                    post().to(workflow_runs_api::task_queue_complete::<Q>),
                )
                .route("/tasks", get().to(workflows_api::tasks::<T>))
                .route("/tasks/{task_id}", get().to(workflows_api::task::<T>))
                .route("/tasks", post().to(workflows_api::create_task::<T>))
                .route("/tasks", post().to(workflows_api::create_task::<T>))
                .route(
                    "/workflow-runs/{workflow_run_id}",
                    get().to(workflow_runs_api::workflow_run::<R>),
                )
                .route(
                    "/workflow-runs",
                    get().to(workflow_runs_api::workflow_runs::<R>),
                )
                .route(
                    "/workflow-runs/init/{workflow_id}",
                    get().to(workflow_runs_api::init_workflow_run::<R>),
                )
                .route(
                    "/workflow-runs/cancel/{workflow_run_id}",
                    get().to(workflow_runs_api::cancel_workflow_run::<R>),
                )
                .route(
                    "/workflow-runs/schedule/{workflow_run_id}",
                    get().to(workflow_runs_api::schedule_workflow_run::<R>),
                )
                .route(
                    "/workflow-runs/restart/{workflow_run_id}",
                    get().to(workflow_runs_api::restart_workflow_run::<R>),
                )
                .route("/workflows", get().to(workflows_api::workflows::<W>))
                .route(
                    "/workflows/{workflow_id}",
                    get().to(workflows_api::workflow::<W>),
                )
                .route("/workflows", post().to(workflows_api::create_workflow::<W>))
                .route(
                    "/workflows",
                    patch().to(workflows_api::update_workflow::<W>),
                )
                .route(
                    "/workflows/deprecate",
                    post().to(workflows_api::deprecate_workflow::<W>),
                ),
        )
    })
    .bind(address)? //("127.0.0.1", 8080))?;
    .run()
    .await?;
    Ok(())
}
