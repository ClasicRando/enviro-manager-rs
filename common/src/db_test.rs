use sqlx::PgPool;
use std::path::PathBuf;
use tokio::fs::read_dir;

use crate::{db_build::db_build, package_dir, workspace_dir, read_file};

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

async fn run_tests(tests_path: PathBuf, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    if !tests_path.exists() {
        return Ok(())
    }

    let mut entries = read_dir(tests_path).await?;
    while let Some(file) = entries.next_entry().await? {
        let block = read_file(file.path()).await?;
        let result = execute_anonymous_block(block, pool).await;
        assert!(
            result.is_ok(),
            "Failed running test in {:?}\n{}",
            file.path(),
            result.unwrap_err()
        )
    }
    Ok(())
}

async fn run_common_db_tests(
    pool: &PgPool,
    common_db_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tests = workspace_dir()
        .join("common-database")
        .join(common_db_name)
        .join("tests");
    run_tests(tests, pool).await
}

pub async fn run_db_tests(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let schema_directory = package_dir().join("database");
    let build_file = schema_directory.join("build.json");
    let db_build = db_build(build_file).await?;
    for common_schema in &db_build.common_dependencies {
        run_common_db_tests(&pool, common_schema).await?;
    }

    let tests = schema_directory.join("tests");
    run_tests(tests, &pool).await
}
