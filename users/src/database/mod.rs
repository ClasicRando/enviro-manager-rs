mod utilities;

pub use utilities::{PostgresConnectionPool, ConnectionPool};

#[cfg(test)]
mod test {
    use common::db_test::run_db_tests;

    use crate::database::{PostgresConnectionPool, ConnectionPool};

    #[tokio::test]
    async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        run_db_tests(&pool).await?;
        Ok(())
    }
}
