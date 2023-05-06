use common::error::EmResult;
use log::{error, info};
use workflow_engine::Executor;

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/executor_log.yml", Default::default()).unwrap();

    info!("Initializing Executor");
    let executor = match Executor::new_postgres().await {
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
