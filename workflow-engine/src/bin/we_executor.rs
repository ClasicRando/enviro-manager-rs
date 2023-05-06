use common::error::EmResult;
use log::{error, info};
use workflow_engine::{
    create_executors_service, create_task_queue_service, create_workflow_runs_service,
    database::{ConnectionPool, PostgresConnectionPool},
    Executor, ExecutorsService, PgExecutorsService, PgTaskQueueService, PgWorkflowRunsService,
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/executor_log.yml", Default::default()).unwrap();

    info!("Initializing Executor");
    let pool = PostgresConnectionPool::create_db_pool().await?;
    let executor_service: PgExecutorsService = create_executors_service(&pool)?;
    let wr_service: PgWorkflowRunsService = create_workflow_runs_service(&pool)?;
    let tq_service: PgTaskQueueService = create_task_queue_service(&pool)?;
    let executor = match Executor::new(&executor_service, &wr_service, &tq_service).await {
        Ok(executor) => executor,
        Err(error) => {
            error!("{}", error);
            return Ok(());
        }
    };
    let executor_id = executor.executor_id().clone();

    info!("Running Executor, id = {}", executor_id);
    if let Err(error) = executor.run().await {
        executor_service.post_error(&executor_id, error).await?;
    }
    info!("Exiting executor, id = {}", executor_id);
    Ok(())
}
