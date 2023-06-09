use common::{
    database::{postgres::Postgres, Database},
    error::EmResult,
};
use workflow_engine::{
    api,
    database::db_options,
    executor::service::postgres::PgExecutorService,
    job::service::postgres::PgJobsService,
    workflow::service::postgres::{PgTasksService, PgWorkflowsService},
    workflow_run::service::postgres::{PgTaskQueueService, PgWorkflowRunsService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/api_server_log.yml", Default::default()).unwrap();
    let options = db_options()?;
    let pool = Postgres::create_pool(options, 20, 1).await?;

    let executor_service = PgExecutorService::new(&pool);
    let task_service = PgTasksService::new(&pool);
    let workflow_service = PgWorkflowsService::new(&pool);
    let workflow_runs_service = PgWorkflowRunsService::new(&pool, &workflow_service);
    let task_queue_service = PgTaskQueueService::new(&pool, &workflow_runs_service);
    let job_service = PgJobsService::new(&pool, &workflow_runs_service);
    api::spawn_api_server(
        executor_service,
        workflow_runs_service,
        task_queue_service,
        task_service,
        workflow_service,
        job_service,
        ("127.0.0.1", 8080),
    )
    .await?;
    Ok(())
}
