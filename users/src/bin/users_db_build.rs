use common::{
    database::{ConnectionPool, PgConnectionPool},
    db_build::build_database,
};

use users::database::{db_options, test_db_options};

///
async fn refresh_test_database() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgConnectionPool::create_db_pool(db_options()?).await?;
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
            PgConnectionPool::create_db_pool(test_db_options()?).await?
        }
        Some(name) if name == "prod" => {
            println!("Target specified as 'prod' to rebuild");
            PgConnectionPool::create_db_pool(db_options()?).await?
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
