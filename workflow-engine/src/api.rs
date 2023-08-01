use std::net::ToSocketAddrs;

use actix_web::{web::Data, App, HttpServer};
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

/// Run generic API server. Creates all the required endpoints and resources. To run the api server,
/// you must have created an [ExecutorService], [WorkflowRunsService], [TaskQueueService],
/// [TaskService], [WorkflowsService] and [JobService] for your desired [Database] implementation.
/// Each component depends on a [Database] type so the system cannot contain disjointed service
/// implementations to operate.
/// # Errors
/// This function will return an error if the server is unable to bind to the specified `address` or
/// the server's `run` method returns an error
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
                .service(executors_api::service::<E>())
                .service(jobs_api::service::<J>())
                .service(workflow_runs_api::task_queue_service::<Q, R>())
                .service(workflow_runs_api::workflow_runs_service::<R>())
                .service(workflows_api::tasks_service::<T>())
                .service(workflows_api::workflows_service::<W>()),
        )
    })
    .bind(address)? //("127.0.0.1", 8080))?;
    .run()
    .await?;
    Ok(())
}
