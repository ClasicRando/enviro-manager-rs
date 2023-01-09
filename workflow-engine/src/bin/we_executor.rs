use log::{error, info};
use workflow_engine::{
    executors_service, task_queue_service, workflow_runs_service, Executor, Result as WEResult,
};

#[tokio::main]
async fn main() -> WEResult<()> {
    log4rs::init_file("workflow-engine/executor_log.yml", Default::default()).unwrap();

    info!("Initializing Executor");
    let executor_service = executors_service().await?;
    let wr_service = workflow_runs_service().await?;
    let tq_service = task_queue_service().await?;
    let mut executor = match Executor::new(executor_service, wr_service, tq_service).await {
        Ok(executor) => executor,
        Err(error) => {
            error!("{}", error);
            return Ok(());
        }
    };

    info!("Running Executor, id = {}", executor.executor_id());
    if let Err(error) = executor.run().await {
        executor_service
            .post_error(executor.executor_id(), error)
            .await?;
    }
    info!("Exiting executor, id = {}", executor.executor_id());
    Ok(())
}
