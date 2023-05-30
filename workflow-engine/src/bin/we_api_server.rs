use common::{error::EmResult, database::postgres::Postgres};
use workflow_engine::{
    api, PgExecutorService, PgJobsService, PgTaskQueueService,
    PgTasksService, PgWorkflowRunsService, PgWorkflowsService,
};
use workflow_engine::database::db_options;

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
