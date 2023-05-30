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
            postgres::{connection::PgConnectionBuilder, test::PgDatabaseTester},
            test::DatabaseTester,
        },
        error::EmResult,
        package_dir, read_file,
    };
    use lazy_regex::{regex, Lazy, Regex};
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
        let tester = PgDatabaseTester::create(&database);
        let test_path = package_dir()?.join("database/tests").join(test_file);
        let block = read_file(test_path).await?;
        let result = tester.execute_anonymous_block_transaction(&block).await;
        if let Some(block_error) = result.block_error {
            panic!("{block_error}")
        }
        if let Some(transaction_error) = result.block_error {
            panic!("{transaction_error}")
        }
        Ok(())
    }

    static ENUM_REGEX: &Lazy<Regex, fn() -> Regex> = regex!(
        r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+)\s+as\s+enum\s*\((?P<labels>[^;]+)\s*\);"
    );
    static COMPOSITE_REGEX: &Lazy<Regex, fn() -> Regex> = regex!(
        r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+?)\s+as\s*\((?P<attributes>[^;]+)\);"
    );

    /// Check a database build unit to see if it defines an enum creation. If it does, it checks to
    /// see if all it's specified labels can be found within the database's definition of the
    /// enum. If the unit is not an enum, then the function exits immediately with an [Ok].
    #[rstest]
    #[case::executor_status("executor/executor_status.pgsql")]
    #[case::job_type("job/job_type.pgsql")]
    #[case::task_status("task/task_status.pgsql")]
    #[case::workflow_run_status("workflow/workflow_run_status.pgsql")]
    #[tokio::test]
    async fn check_for_enum(database: PgPool, #[case] file_name: &str) -> EmResult<()> {
        let file_path = package_dir()?.join("database").join(file_name);
        let block = read_file(file_path).await?;
        let Some(captures) = ENUM_REGEX.captures(&block) else {
            Err("Provided file does not match the ENUM_REGEX pattern")?
        };
        let Some(schema) = captures.name("schema") else {
            Err("No 'schema' capture group present in enum definition")?
        };
        let Some(name) = captures.name("name") else {
            Err("No 'name' capture group present in enum definition")?
        };
        let Some(labels) = captures.name("labels") else {
            Err("No 'labels' capture group present in enum definition")?
        };
        let labels: Vec<&str> = labels
            .as_str()
            .split(',')
            .map(|label| label.trim().trim_matches('\''))
            .collect();
        sqlx::query("call data_check.check_enum_definition($1,$2,$3)")
            .bind(schema.as_str())
            .bind(name.as_str())
            .bind(labels)
            .execute(&database)
            .await?;
        Ok(())
    }

    /// Check a database build unit to see if it defines a composite creation. If it does, it checks
    /// to see if all it's specified attributes can be found within the database's definition of
    /// the composite. If the unit is not a composite, then the function exits immediately with
    /// an [Ok].
    #[rstest]
    #[case("job/schedule_entry.pgsql")]
    #[case("task/task_rule.pgsql")]
    #[case("task/workflow_run_task.pgsql")]
    #[case("task/workflow_task.pgsql")]
    #[case("task/workflow_task_request.pgsql")]
    #[tokio::test]
    async fn check_for_composite(database: PgPool, #[case] file_name: &str) -> EmResult<()> {
        let file_path = package_dir()?.join("database").join(file_name);
        let block = read_file(file_path).await?;
        let Some(captures) = COMPOSITE_REGEX.captures(&block) else {
            return Ok(())
        };
        let Some(schema) = captures.name("schema") else {
            Err("No 'schema' capture group present in composite definition")?
        };
        let Some(name) = captures.name("name") else {
            Err("No 'name' capture group present in composite definition")?
        };
        let Some(attributes) = captures.name("attributes") else {
            Err("No 'attributes' capture group present in composite definition")?
        };
        let attributes: Vec<&str> = attributes
            .as_str()
            .split(',')
            .map(|label| label.trim())
            .collect();
        sqlx::query("call data_check.check_composite_definition($1,$2,$3)")
            .bind(schema.as_str())
            .bind(name.as_str())
            .bind(attributes)
            .execute(&database)
            .await?;
        Ok(())
    }
}
