use common::db_build::build_database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = workflow_engine::database::we_db_pool().await?;
    build_database(pool).await?;
    Ok(())
}
