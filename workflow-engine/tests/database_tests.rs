use common::db_test::run_db_tests;
use workflow_engine::database::{create_db_pool, create_test_db_pool};

async fn refresh_test_database() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_db_pool().await?;
    sqlx::query("drop database if exists workflow_engine_test")
        .execute(&pool)
        .await?;
    sqlx::query(
        r#"
        create database workflow_engine_test with
            owner = workflow_engine_admin
            encoding = 'UTF8'"#,
    )
    .execute(&pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
    let db_refresh = std::env::var("WE_TEST_REFRESH")
        .map(|v| &v == "true")
        .unwrap_or_default();
    if db_refresh {
        refresh_test_database().await?;
    }
    
    let pool = create_test_db_pool().await?;
    run_db_tests(pool).await?;
    Ok(())
}
