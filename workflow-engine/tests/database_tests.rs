use common::db_test::run_db_tests;
use workflow_engine::database::create_test_db_pool;

#[tokio::test]
async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_test_db_pool().await?;
    run_db_tests(pool).await?;
    Ok(())
}
