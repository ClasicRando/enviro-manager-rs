use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;
use std::path::PathBuf;
use tokio::fs::read_dir;

use crate::{db_build::db_build, execute_anonymous_block, package_dir, read_file, workspace_dir};

async fn run_tests(tests_path: PathBuf, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    if !tests_path.exists() {
        return Ok(());
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

lazy_static! {
    static ref ENUM_REGEX: Regex = Regex::new(
        r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+)\s+as\s+enum\s*\((?P<labels>[^;]+)\s*\);"
    )
    .unwrap();
    static ref COMPOSITE_REGEX: Regex = Regex::new(
        r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+?)\s+as\s*\((?P<attributes>[^;]+)\);"
    )
    .unwrap();
}

async fn check_for_enum(block: &str, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let Some(captures) = ENUM_REGEX.captures(block) else {
        return Ok(())
    };
    let Some(schema) = captures.name("schema") else {
        Err(format!("No 'schema' capture group present in enum definition"))?
    };
    let Some(name) = captures.name("name") else {
        Err(format!("No 'name' capture group present in enum definition"))?
    };
    let Some(labels) = captures.name("labels") else {
        Err(format!("No 'labels' capture group present in enum definition"))?
    };
    let labels: Vec<&str> = labels
        .as_str()
        .split(',')
        .map(|label| label.trim().trim_matches('\''))
        .collect();
    sqlx::query("call data_check.check_enum_definition($1,$2,$3)")
        .bind(schema.as_str())
        .bind(name.as_str())
        .bind(labels)
        .execute(pool)
        .await?;
    Ok(())
}

async fn check_for_composite(block: &str, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let Some(captures) = COMPOSITE_REGEX.captures(block) else {
        return Ok(())
    };
    let Some(schema) = captures.name("schema") else {
        Err(format!("No 'schema' capture group present in composite definition"))?
    };
    let Some(name) = captures.name("name") else {
        Err(format!("No 'name' capture group present in composite definition"))?
    };
    let Some(attributes) = captures.name("attributes") else {
        Err(format!("No 'attributes' capture group present in composite definition"))?
    };
    let attributes: Vec<&str> = attributes
        .as_str()
        .split(',')
        .map(|label| label.trim())
        .collect();
    sqlx::query("call data_check.check_composite_definition($1,$2,$3)")
        .bind(schema.as_str())
        .bind(name.as_str())
        .bind(attributes)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn run_db_tests(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let schema_directory = package_dir().join("database");
    let build_file = schema_directory.join("build.json");
    let db_build = db_build(build_file).await?;
    for common_schema in &db_build.common_dependencies {
        run_common_db_tests(&pool, common_schema).await?;
    }

    for entry in db_build.entries {
        let block = read_file(schema_directory.join(&entry.name)).await?;
        check_for_enum(&block, &pool).await?;
        check_for_composite(&block, &pool).await?;
    }

    let tests = schema_directory.join("tests");
    run_tests(tests, &pool).await
}
