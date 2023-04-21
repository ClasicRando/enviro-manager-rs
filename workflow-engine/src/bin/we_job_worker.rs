use log::{error, info};
use workflow_engine::{create_jobs_service, JobWorker, Result as WEResult, database::create_db_pool};

#[tokio::main]
async fn main() -> WEResult<()> {
    log4rs::init_file("workflow-engine/job_worker_log.yml", Default::default()).unwrap();

    info!("Initializing Worker");
    let pool = create_db_pool().await?;
    let jobs_service = create_jobs_service(&pool)?;
    let worker = match JobWorker::new(jobs_service).await {
        Ok(worker) => worker,
        Err(error) => {
            error!("{}", error);
            return Err(error);
        }
    };

    info!("Running Worker");
    if let Err(error) = worker.run().await {
        error!("Error during worker run\n{}", error)
    }

    info!("Exiting Worker");
    Ok(())
}
