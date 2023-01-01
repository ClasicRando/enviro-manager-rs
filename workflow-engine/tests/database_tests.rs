use sqlx::PgPool;
use std::path::PathBuf;
use tokio::{
    fs::{read_dir, File},
    io::AsyncReadExt,
};

use workflow_engine::database::create_db_pool;

fn get_relative_path(
    path: &str,
    from_workspace: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let dir = if from_workspace {
        match cwd.parent() {
            Some(workspace) => workspace,
            None => return Err("Could not find cwd parent directory".into()),
        }
    } else {
        cwd.as_path()
    };
    let mut result = PathBuf::from(dir);
    result.push(path.trim_start_matches('/'));
    Ok(result)
}

async fn execute_anonymous_block(block: String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let block = match block.split_whitespace().next() {
        Some("do") => block,
        Some("begin" | "declare") => format!("do $body$\n{}\n$body$;", block),
        Some(_) => format!("do $body$\nbegin\n{}\nend;\n$body$;", block),
        None => block,
    };
    sqlx::query(&block).execute(pool).await?;
    Ok(())
}

async fn read_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path).await?;
    let mut block = String::new();
    file.read_to_string(&mut block).await?;
    Ok(block)
}

#[tokio::test]
async fn run_data_check_database_tests() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_db_pool().await?;
    let tests = get_relative_path("/common-database/data_check/tests", true)?;
    let mut entries = read_dir(tests).await?;
    while let Some(file) = entries.next_entry().await? {
        let block = read_file(file.path()).await?;
        let result = execute_anonymous_block(block, &pool).await;
        assert!(
            result.is_ok(),
            "Failed running test in {:?}\n{}",
            file.path(),
            result.unwrap_err()
        )
    }
    Ok(())
}

#[tokio::test]
async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_db_pool().await?;
    let tests = get_relative_path("/database/tests", false)?;
    let mut entries = read_dir(tests).await?;
    while let Some(file) = entries.next_entry().await? {
        let block = read_file(file.path()).await?;
        let result = execute_anonymous_block(block, &pool).await;
        assert!(
            result.is_ok(),
            "Failed running test in {:?}\n{}",
            file.path(),
            result.unwrap_err()
        )
    }
    Ok(())
}
