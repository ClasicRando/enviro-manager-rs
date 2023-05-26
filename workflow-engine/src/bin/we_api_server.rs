use common::error::EmResult;
use sqlx::Postgres;
use workflow_engine::{
    api, database::PostgresConnectionPool, PgExecutorService, PgJobsService, PgTaskQueueService,
    PgTasksService, PgWorkflowRunsService, PgWorkflowsService,
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/api_server_log.yml", Default::default()).unwrap();
    api::spawn_api_server::<
        (&str, u16),
        PostgresConnectionPool,
        Postgres,
        PgExecutorService,
        PgJobsService,
        PgTaskQueueService,
        PgWorkflowRunsService,
        PgTasksService,
        PgWorkflowsService,
    >(("127.0.0.1", 8080))
    .await?;
    Ok(())
}
