use common::db_build::build_schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = workflow_engine::database::create_db_pool().await?;
    build_schema(&pool).await?;
    if let Ok(db) = std::env::var("WE_TEST_DB") {
        if db.trim().is_empty() {
            return Ok(())
        }
        let pool = workflow_engine::database::create_test_db_pool().await?;
        build_schema(&pool).await?;
    }
    Ok(())
}
