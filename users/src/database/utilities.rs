use std::env;

use common::error::EmResult;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

/// Return database connect options
fn db_options() -> EmResult<PgConnectOptions> {
    let port = env::var("EMU_PORT")?
        .parse()
        .expect("Port environment variable is not an integer");
    let options = PgConnectOptions::new()
        .host(&env::var("EMU_HOST")?)
        .port(port)
        .database(&env::var("EMU_DB")?)
        .username(&env::var("EMU_USER")?)
        .password(&env::var("EMU_PASSWORD")?);
    Ok(options)
}

/// Return test database connect options
fn test_db_options() -> EmResult<PgConnectOptions> {
    let port = env::var("EMU_TEST_PORT")?
        .parse()
        .expect("Port environment variable is not an integer");
    let options = PgConnectOptions::new()
        .host(&env::var("EMU_TEST_HOST")?)
        .port(port)
        .database(&env::var("EMU_TEST_DB")?)
        .username(&env::var("EMU_TEST_USER")?)
        .password(&env::var("EMU_TEST_PASSWORD")?);
    Ok(options)
}

/// Return a new pool of postgres connections
pub async fn create_db_pool() -> EmResult<PgPool> {
    let options = db_options()?;
    let pool = PgPoolOptions::new()
        .min_connections(10)
        .max_connections(20)
        .connect_with(options)
        .await?;
    Ok(pool)
}

/// Return a new pool of postgres connections for the test database
pub async fn create_test_db_pool() -> EmResult<PgPool> {
    let options = test_db_options()?;
    let pool = PgPoolOptions::new().connect_with(options).await?;
    Ok(pool)
}
