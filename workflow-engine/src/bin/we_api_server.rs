use common::{database::postgres::Postgres, error::EmResult};
use workflow_engine::{
    api,
    database::db_options,
    job::service::postgres::PgJobsService,
    services::postgres::{
        executors::PgExecutorService, tasks::PgTasksService, workflows::PgWorkflowsService,
    },
    workflow_run::service::postgres::{PgTaskQueueService, PgWorkflowRunsService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/api_server_log.yml", Default::default()).unwrap();
    api::spawn_api_server::<
        (&str, u16),
        Postgres,
        PgExecutorService,
        PgJobsService,
        PgTaskQueueService,
        PgWorkflowRunsService,
        PgTasksService,
        PgWorkflowsService,
    >(("127.0.0.1", 8080), db_options()?)
    .await?;
    Ok(())
}
