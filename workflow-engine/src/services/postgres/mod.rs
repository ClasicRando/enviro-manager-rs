pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

#[cfg(test)]
pub(crate) mod test {
    use common::{
        database::{
            connection::ConnectionBuilder,
            postgres::{connection::PgConnectionBuilder},
        },
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
    #[case::valid_job_schedule("job/valid_job_schedule.pgsql")]
    #[case::are_valid_task_rules("task/are_valid_task_rules.pgsql")]
    #[case::workflow_tasks("task/workflow_tasks.pgsql")]
    #[tokio::test]
    async fn database_test(database: PgPool, #[case] test_file: &str) -> EmResult<()> {
        common::database::postgres::test::run_db_test(&database, test_file).await
    }

    #[rstest]
    #[case::executor_status("executor/executor_status.pgsql")]
    #[case::job_type("job/job_type.pgsql")]
    #[case::task_status("task/task_status.pgsql")]
    #[case::workflow_run_status("workflow/workflow_run_status.pgsql")]
    #[tokio::test]
    async fn check_for_enum(database: PgPool, #[case] file_name: &str) -> EmResult<()> {
        common::database::postgres::test::check_for_enum(&database, file_name).await
    }

    #[rstest]
    #[case("job/schedule_entry.pgsql")]
    #[case("task/task_rule.pgsql")]
    #[case("task/workflow_run_task.pgsql")]
    #[case("task/workflow_task.pgsql")]
    #[case("task/workflow_task_request.pgsql")]
    #[tokio::test]
    async fn check_for_composite(database: PgPool, #[case] file_name: &str) -> EmResult<()> {
        common::database::postgres::test::check_for_composite(&database, file_name).await
    }
}
