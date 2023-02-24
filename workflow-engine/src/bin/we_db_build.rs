use common::db_build::build_schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = workflow_engine::database::create_db_pool().await?;
    build_schema("/workflow-engine/database", &pool).await?;
    Ok(())
}
