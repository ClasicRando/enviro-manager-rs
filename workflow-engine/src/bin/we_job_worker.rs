use common::error::EmResult;
use log::{error, info};
use workflow_engine::{
    database::{ConnectionPool, PostgresConnectionPool},
    JobWorker, JobsService, PgJobsService, PgWorkflowRunsService, WorkflowRunsService,
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/job_worker_log.yml", Default::default()).unwrap();

    info!("Initializing Worker");
    let pool = PostgresConnectionPool::create_db_pool().await?;
    let workflow_runs_service = PgWorkflowRunsService::new(&pool);
    let jobs_service = PgJobsService::create(&pool, &workflow_runs_service);
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
