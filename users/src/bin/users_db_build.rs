use std::path::PathBuf;

use common::{
    database::{ConnectionBuilder, PgConnectionBuilder},
    db_build::build_database,
};
use log::{error, info, warn};
use sqlx::{PgPool, postgres::PgConnectOptions};
use users::database::{db_options, test_db_options};

/// Drop the current test database and create a new version to be populated by the [build_database]
/// method.
async fn refresh_test_database(options: PgConnectOptions) -> Result<PgPool, Box<dyn std::error::Error>> {
    let admin_pool: PgPool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
    sqlx::query("drop database if exists em_user_test")
        .execute(&admin_pool)
        .await?;
    sqlx::query(
        r#"
        create database em_user_test with
            owner = users_test
            encoding = 'UTF8'"#,
    )
    .execute(&admin_pool)
    .await?;
    let test_pool: PgPool = PgConnectionBuilder::create_pool(options, 1, 1).await?;
    sqlx::query("create extension if not exists pgcrypto")
        .execute(&test_pool)
        .await?;
    Ok(test_pool)
}

#[tokio::main]
async fn main() {
    log4rs::init_file("users/users_db_build_log.yml", Default::default()).unwrap();
    let db_refresh = std::env::var("USERS_TEST_REFRESH")
        .map(|v| &v == "true")
        .unwrap_or_default();

    let Some(name) = std::env::var("USERS_DB_BUILD_TARGET").ok() else {
        warn!(
            "Could not find a value for the database build target. Please specify with the \
             'USERS_DB_BUILD_TARGET' environment variable"
        );
        return;
    };
    let pool_result = match name.as_str() {
        "test" => {
            info!("Target specified as 'test' to rebuild");
            let options = match test_db_options() {
                Ok(inner) => inner,
                Err(error) => {
                    error!("Error getting test database options. {}", error);
                    return;
                }
            };
            if db_refresh {
                info!("Refresh specified for the test database");
                match refresh_test_database(options).await {
                    Ok(pool) => Ok(pool),
                    Err(error) => {
                        error!("Error refreshing test database. {}", error);
                        return;
                    }
                }
            } else {
                PgConnectionBuilder::create_pool(options, 1, 1).await
            }
        }
        "prod" => {
            info!("Target specified as 'prod' to rebuild");
            let options = match db_options() {
                Ok(inner) => inner,
                Err(error) => {
                    error!("Error getting prod database options. {}", error);
                    return;
                }
            };
            PgConnectionBuilder::create_pool(options, 1, 1).await
        }
        _ => {
            warn!(
                "Target specified in 'USERS_DB_BUILD_TARGET' environment variable ('{}') was not \
                 valid. Acceptable values are 'test' or 'prod'",
                name
            );
            return;
        }
    };
    let pool = match pool_result {
        Ok(inner) => inner,
        Err(error) => {
            error!("Error getting database connection pool. {}", error);
            return;
        }
    };

    if let Err(error) = build_database(&pool).await {
        error!("Error building {} database. {}", name, error);
        return;
    }

    if name == "test" {
        let test_scaffold_path = PathBuf::from("./users/database/test_scaffold.pgsql");

        if !test_scaffold_path.exists() {
            info!("No test scaffold file. Continuing")
        }

        let block_result = match common::read_file(&test_scaffold_path).await {
            Ok(sql_block) => common::execute_anonymous_block(&sql_block, &pool).await,
            Err(error) => {
                error!("Error scaffolding test database. {}", error);
                return;
            }
        };
        if let Err(error) = block_result {
            error!("Error scaffolding test database. {}", error)
        }
    }
}
