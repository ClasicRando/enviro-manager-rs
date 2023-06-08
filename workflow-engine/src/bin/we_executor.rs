use common::{
    database::{postgres::Postgres, Database},
    error::EmResult,
};
use log::{error, info};
use workflow_engine::{
    database::db_options,
    executor::{service::postgres::PgExecutorService, worker::Executor},
    workflow_run::service::postgres::{PgTaskQueueService, PgWorkflowRunsService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/executor_log.yml", Default::default()).unwrap();

    info!("Initializing Executor");
    let options = db_options()?;
    let pool = Postgres::create_pool(options, 20, 1).await?;
    let executor_service = PgExecutorService::new(&pool);
    let wr_service = PgWorkflowRunsService::new(&pool);
    let tq_service = PgTaskQueueService::new(&pool, &wr_service);
    let executor = match Executor::new(&executor_service, &wr_service, &tq_service).await {
        Ok(executor) => executor,
        Err(error) => {
            error!("{}", error);
            return Ok(());
        }
    };
    let executor_id = executor.executor_id().clone();

    info!("Running Executor, id = {}", executor_id);
    executor.run().await;
    info!("Exiting executor, id = {}", executor_id);
    Ok(())
}
