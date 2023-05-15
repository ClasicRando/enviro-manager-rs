use std::path::PathBuf;

use common::{
    database::{ConnectionBuilder, PgConnectionBuilder},
    db_build::build_database,
};

use users::database::{db_options, test_db_options};

///
async fn refresh_test_database() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
    sqlx::query("drop database if exists em_user_test")
        .execute(&pool)
        .await?;
    sqlx::query(
        r#"
        create database em_user_test with
            owner = users_admin
            encoding = 'UTF8'"#,
    )
    .execute(&pool)
    .await?;

    let pool = PgConnectionBuilder::create_pool(test_db_options()?, 1, 1).await?;
    sqlx::query(
        r#"
        create extension pgcrypto;
        grant all on schema users to users_test;
        grant all on schema data_check to users_test;
        grant all on schema audit to users_test;
        grant all on all tables in schema users to users_test;
        grant all on all procedures in schema users to users_test;
        grant all on all functions in schema users to users_test;
        grant all on all functions in schema data_check to users_test;
        grant all on all tables in schema audit to users_test;
        grant all on all procedures in schema audit to users_test;
        grant all on all functions in schema audit to users_test;"#,
    )
    .execute(&pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_refresh = std::env::var("USERS_TEST_REFRESH")
        .map(|v| &v == "true")
        .unwrap_or_default();

    let Some(name) = std::env::var("USERS_DB_BUILD_TARGET").ok() else {
        println!(
            "Could not find a value for the database build target. Please specify with the \
             'USERS_DB_BUILD_TARGET' environment variable"
        );
        return Ok(());
    };
    let pool = match name.as_str() {
        "test" => {
            println!("Target specified as 'test' to rebuild");
            if db_refresh {
                println!("Refresh specified for the test database");
                refresh_test_database().await?;
            }
            PgConnectionBuilder::create_pool(test_db_options()?, 1, 1).await?
        }
        "prod" => {
            println!("Target specified as 'prod' to rebuild");
            PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?
        }
        _ => {
            println!(
                "Target specified in 'USERS_DB_BUILD_TARGET' environment variable ('{}') was not \
                 valid. Acceptable values are 'test' or 'prod'",
                name
            );
            return Ok(());
        }
    };
    build_database(&pool).await?;

    if name == "test" {
        let test_scaffold_path = PathBuf::from("./users/database/test_scaffold.pgsql");
        if let Ok(sql_block) = common::read_file(&test_scaffold_path).await {
            common::execute_anonymous_block(&sql_block, &pool).await?;
        }
    }
    
    Ok(())
}
