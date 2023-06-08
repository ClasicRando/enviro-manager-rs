use common::{
    database::{connection::ConnectionBuilder, postgres::connection::PgConnectionBuilder},
    email::ClippyEmailService,
    error::EmResult,
};
use log::{error, info};
use workflow_engine::{
    database::db_options,
    job::service::{postgres::PgJobsService, JobService},
    job_worker::JobWorker,
    workflow_run::service::{postgres::PgWorkflowRunsService, WorkflowRunsService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/job_worker_log.yml", Default::default()).unwrap();

    info!("Initializing Worker");
    let pool = PgConnectionBuilder::create_pool(db_options()?, 20, 1).await?;
    let workflow_runs_service = PgWorkflowRunsService::create(&pool);
    let jobs_service = PgJobsService::create(&pool, &workflow_runs_service);
    let email_service = ClippyEmailService::new()?;
    let worker = match JobWorker::new(jobs_service, email_service) {
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
