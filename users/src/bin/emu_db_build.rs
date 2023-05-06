use common::db_build::build_database;
use users::database::{create_db_pool, create_test_db_pool};

///
async fn refresh_test_database() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_db_pool().await?;
    sqlx::query("drop database if exists enviro_manager_user_test")
        .execute(&pool)
        .await?;
    sqlx::query(
        r#"
        create database enviro_manager_user_test with
            owner = emu_admin
            encoding = 'UTF8'"#,
    )
    .execute(&pool)
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_refresh = std::env::var("EMU_TEST_REFRESH")
        .map(|v| &v == "true")
        .unwrap_or_default();

    let pool = match std::env::var("EMU_DB_BUILD_TARGET").ok() {
        Some(name) if name == "test" => {
            println!("Target specified as 'test' to rebuild");
            if db_refresh {
                println!("Refresh specified for the test database");
                refresh_test_database().await?;
            }
            create_test_db_pool().await?
        }
        Some(name) if name == "prod" => {
            println!("Target specified as 'prod' to rebuild");
            create_db_pool().await?
        }
        Some(name) => {
            println!(
                "Target specified in 'EMU_DB_BUILD_TARGET' environment variable ('{}') was not \
                 valid. Acceptable values are 'test' or 'prod'",
                name
            );
            return Ok(());
        }
        None => {
            println!(
                "Could not find a value for the database build target. Please specify with the \
                 'EMU_DB_BUILD_TARGET' environment variable"
            );
            return Ok(());
        }
    };
    build_database(&pool).await?;
    Ok(())
}
