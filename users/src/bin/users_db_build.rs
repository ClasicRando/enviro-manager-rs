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
        create extension pgcrypto"#,
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

    let pool = match std::env::var("USERS_DB_BUILD_TARGET").ok() {
        Some(name) if name == "test" => {
            println!("Target specified as 'test' to rebuild");
            if db_refresh {
                println!("Refresh specified for the test database");
                refresh_test_database().await?;
            }
            PgConnectionBuilder::create_pool(test_db_options()?, 1, 1).await?
        }
        Some(name) if name == "prod" => {
            println!("Target specified as 'prod' to rebuild");
            PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?
        }
        Some(name) => {
            println!(
                "Target specified in 'USERS_DB_BUILD_TARGET' environment variable ('{}') was not \
                 valid. Acceptable values are 'test' or 'prod'",
                name
            );
            return Ok(());
        }
        None => {
            println!(
                "Could not find a value for the database build target. Please specify with the \
                 'USERS_DB_BUILD_TARGET' environment variable"
            );
            return Ok(());
        }
    };
    build_database(&pool).await?;
    Ok(())
}
