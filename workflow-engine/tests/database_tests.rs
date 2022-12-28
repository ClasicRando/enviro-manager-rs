use std::path::PathBuf;
use sqlx::PgPool;
use tokio::{fs::{File, read_dir}, io::AsyncReadExt};

use workflow_engine::create_we_db_pool;

fn get_path_from_workspace(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let Some(workspace) = cwd.parent() else {
        return Err("Could not find cwd parent directory".into())
    };
    let mut result = PathBuf::from(workspace);
    result.push(path);
    Ok(result)
}

async fn execute_anonymous_block(block: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(block)
        .execute(pool)
        .await?;
    Ok(())
}

async fn read_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path).await?;
    let mut block = String::new();
    file.read_to_string(&mut block).await?;
    Ok(block)
}

#[tokio::test]
async fn run_database_tests() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_we_db_pool().await?;
    let tests = get_path_from_workspace("common-database/data_check/tests")?;
    let mut entries = read_dir(tests).await?;
    while let Some(file) = entries.next_entry().await? {
        let block = read_file(file.path()).await?;
        execute_anonymous_block(&block, &pool).await?;
    }
    Ok(())
}