use common::{database::postgres::Postgres, error::EmResult};
use workflow_engine::{
    api,
    database::db_options,
    services::postgres::{
        executors::PgExecutorService, jobs::PgJobsService, task_queue::PgTaskQueueService,
        tasks::PgTasksService, workflow_runs::PgWorkflowRunsService, workflows::PgWorkflowsService,
    },
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
