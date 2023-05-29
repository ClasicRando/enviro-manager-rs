use std::net::ToSocketAddrs;

use actix_web::{
    web::{get, post, Data},
    App, HttpServer,
};
use common::{database::connection::ConnectionBuilder, error::EmResult};
use sqlx::{Connection, Database};

pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

use crate::{
    ExecutorService, JobService, TaskQueueService, TaskService, WorkflowRunsService,
    WorkflowsService,
};

pub async fn spawn_api_server<A, C, D, E, J, Q, R, T, W>(
    address: A,
    options: <D::Connection as Connection>::Options,
) -> EmResult<()>
where
    A: ToSocketAddrs,
    C: ConnectionBuilder<D>,
    D: Database,
    E: ExecutorService<Database = D> + Send + Sync + 'static,
    J: JobService<Database = D, WorkflowRunService = R> + Send + Sync + 'static,
    Q: TaskQueueService<Database = D, WorkflowRunService = R> + Send + Sync + 'static,
    R: WorkflowRunsService<Database = D> + Send + Sync + 'static,
    T: TaskService<Database = D> + Send + Sync + 'static,
    W: WorkflowsService<Database = D> + Send + Sync + 'static,
{
    let pool = C::create_pool(options, 20, 1).await?;
    let executors_service: Data<E> = Data::new(E::create(&pool));
    let workflow_runs_service: Data<R> = Data::new(R::create(&pool));
    let task_queue_service: Data<Q> = Data::new(Q::create(&pool, workflow_runs_service.as_ref()));
    let tasks_service: Data<T> = Data::new(T::create(&pool));
    let workflows_service: Data<W> = Data::new(W::create(&pool));
    let jobs_service: Data<J> = Data::new(J::create(&pool, workflow_runs_service.get_ref()));
    HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .app_data(executors_service.clone())
                .app_data(jobs_service.clone())
                .app_data(task_queue_service.clone())
                .app_data(tasks_service.clone())
                .app_data(workflow_runs_service.clone())
                .app_data(workflows_service.clone())
                .route("/executors", get().to(executors::active_executors::<E>))
                .route(
                    "/executors/shutdown/{executor_id}",
                    get().to(executors::shutdown_executor::<E>),
                )
                .route(
                    "/executors/cancel/{executor_id}",
                    get().to(executors::cancel_executor::<E>),
                )
                .route("/jobs", get().to(jobs::jobs::<J>))
                .route("/jobs/{job_id}", get().to(jobs::job::<J>))
                .route("/jobs", post().to(jobs::create_job::<J>))
                .route(
                    "/task-queue/retry",
                    post().to(task_queue::task_queue_retry::<Q>),
                )
                .route(
                    "/task-queue/complete",
                    post().to(task_queue::task_queue_complete::<Q>),
                )
                .route("/tasks", get().to(tasks::tasks::<T>))
                .route("/tasks/{task_id}", get().to(tasks::task::<T>))
                .route("/tasks", post().to(tasks::create_task::<T>))
                .route("/tasks", post().to(tasks::create_task::<T>))
                .route(
                    "/workflow_runs/{workflow_run_id}",
                    get().to(workflow_runs::workflow_run::<R>),
                )
                .route(
                    "/workflow_runs/init/{workflow_id}",
                    get().to(workflow_runs::init_workflow_run::<R>),
                )
                .route(
                    "/workflow_runs/cancel/{workflow_run_id}",
                    get().to(workflow_runs::cancel_workflow_run::<R>),
                )
                .route(
                    "/workflow_runs/schedule/{workflow_run_id}",
                    get().to(workflow_runs::schedule_workflow_run::<R>),
                )
                .route(
                    "/workflow_runs/restart/{workflow_run_id}",
                    get().to(workflow_runs::restart_workflow_run::<R>),
                )
                .route("/workflows", get().to(workflows::workflows::<W>))
                .route(
                    "/workflows/{workflow_id}",
                    get().to(workflows::workflow::<W>),
                )
                .route("/workflows", post().to(workflows::create_workflow::<W>))
                .route(
                    "/workflows/deprecate",
                    post().to(workflows::deprecate_workflow::<W>),
                ),
        )
    })
    .bind(address)? //("127.0.0.1", 8080))?;
    .run()
    .await?;
    Ok(())
}
