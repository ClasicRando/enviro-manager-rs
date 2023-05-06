use std::net::ToSocketAddrs;

use common::error::EmResult;
use sqlx::Database;

pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod utilities;
pub mod workflow_runs;
pub mod workflows;

use actix_web::{
    web::{get, Data},
    App, HttpServer,
};

use crate::{
    api, create_executors_service, create_jobs_service, create_task_queue_service,
    create_tasks_service, create_workflow_runs_service, create_workflows_service,
    database::ConnectionPool, ExecutorsService, JobsService, TaskQueueService, TasksService,
    WorkflowRunsService, WorkflowsService,
};

pub async fn spawn_api_server<A, C, D, E, J, Q, R, T, W>(address: A) -> EmResult<()>
where
    A: ToSocketAddrs,
    C: ConnectionPool<D>,
    D: Database,
    E: ExecutorsService<Database = D> + Send + Sync + 'static,
    J: JobsService<Database = D> + Send + Sync + 'static,
    Q: TaskQueueService<Database = D> + Send + Sync + 'static,
    R: WorkflowRunsService<Database = D> + Send + Sync + 'static,
    T: TasksService<Database = D> + Send + Sync + 'static,
    W: WorkflowsService<Database = D> + Send + Sync + 'static,
{
    let pool = C::create_db_pool().await?;
    let executors_service: Data<E> = Data::new(create_executors_service::<E, D>(&pool)?);
    let jobs_service: Data<J> = Data::new(create_jobs_service::<J, D>(&pool)?);
    let task_queue_service: Data<Q> = Data::new(create_task_queue_service::<Q, D>(&pool)?);
    let tasks_service: Data<T> = Data::new(create_tasks_service::<T, D>(&pool)?);
    let workflow_runs_service: Data<R> = Data::new(create_workflow_runs_service::<R, D>(&pool)?);
    let workflows_service: Data<W> = Data::new(create_workflows_service::<W, D>(&pool)?);
    HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .app_data(executors_service.clone())
                .app_data(jobs_service.clone())
                .app_data(task_queue_service.clone())
                .app_data(tasks_service.clone())
                .app_data(workflow_runs_service.clone())
                .app_data(workflows_service.clone())
                .route(
                    "/executors",
                    get().to(api::executors::active_executors::<E>),
                )
                .route(
                    "/executors/shutdown/{executor_id}",
                    get().to(api::executors::shutdown_executor::<E>),
                )
                .route(
                    "/executors/cancel/{executor_id}",
                    get().to(api::executors::cancel_executor::<E>),
                ),
        )
    })
    .bind(address)? //("127.0.0.1", 8080))?;
    .run()
    .await?;
    Ok(())
}
