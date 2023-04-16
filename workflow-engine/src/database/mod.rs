pub(crate) mod utilities;

pub use utilities::{we_db_pool, we_test_db_pool};

#[cfg(test)]
mod test {
    use crate::database::utilities::create_test_db_pool;
    use common::db_test::run_db_tests;

    #[tokio::test]
    async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        run_db_tests(&pool).await?;
        Ok(())
    }
}
