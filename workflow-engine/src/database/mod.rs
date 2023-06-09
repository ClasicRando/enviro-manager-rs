use std::env;

use common::error::EmResult;
use sqlx::postgres::PgConnectOptions;

/// Return database connect options
/// # Errors
/// This function returns an error if any of the required environment variables are not present or
/// the port environment variable cannot be parsed as a [u16]. The environment variables required
/// are:
/// - WE_HOST -> address to the postgres database server
/// - WE_PORT -> port that the postgres database is listening
/// - WE_DB -> name of the database to connect
/// - WE_USER -> name of the user to connect as
/// - WE_PASSWORD -> password of the user to connect as
pub fn db_options() -> EmResult<PgConnectOptions> {
    let port = env::var("WE_PORT")?.parse()?;
    let options = PgConnectOptions::new()
        .host(&env::var("WE_HOST")?)
        .port(port)
        .database(&env::var("WE_DB")?)
        .username(&env::var("WE_USER")?)
        .password(&env::var("WE_PASSWORD")?);
    Ok(options)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
pub(crate) mod test {
    use common::{
        database::{connection::ConnectionBuilder, postgres::connection::PgConnectionBuilder},
        error::EmResult,
    };
    use rstest::{fixture, rstest};
    use sqlx::PgPool;

    use crate::database::db_options;

    #[fixture]
    pub(crate) fn database() -> PgPool {
        let options = db_options().expect("Failed to create test database options");
        PgConnectionBuilder::create_pool_lazy(options, 1, 1)
    }

    #[rstest]
    #[case::clean_executors("executor/clean_executors.pgsql")]
    #[case::next_run_job_schedule("job/next_run_job_schedule.pgsql")]
    #[tokio::test]
    async fn database_test(database: PgPool, #[case] test_file: &str) -> EmResult<()> {
        common::database::postgres::test::run_db_test(&database, test_file).await
    }

    #[rstest]
    #[case::executor_status("executor/executor_status.pgsql")]
    #[case::job_type("job/job_type.pgsql")]
    #[case::task_status("workflow_run/task_status.pgsql")]
    #[case::workflow_run_status("workflow_run/workflow_run_status.pgsql")]
    #[tokio::test]
    async fn check_for_enum(database: PgPool, #[case] file_name: &str) -> EmResult<()> {
        common::database::postgres::test::check_for_enum(&database, file_name).await
    }

    #[rstest]
    #[case("job/schedule_entry.pgsql")]
    #[case("workflow_run/task_rule.pgsql")]
    #[case("workflow_run/workflow_run_task.pgsql")]
    #[case("workflow/workflow_task.pgsql")]
    #[case("workflow/workflow_task_request.pgsql")]
    #[tokio::test]
    async fn check_for_composite(database: PgPool, #[case] file_name: &str) -> EmResult<()> {
        common::database::postgres::test::check_for_composite(&database, file_name).await
    }
}
