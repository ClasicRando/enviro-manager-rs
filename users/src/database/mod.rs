pub mod utilities;

pub use utilities::{create_db_pool, create_test_db_pool};

#[cfg(test)]
mod test {
    use common::db_test::run_db_tests;

    use crate::database::utilities::create_test_db_pool;

    #[tokio::test]
    async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        run_db_tests(&pool).await?;
        Ok(())
    }
}
