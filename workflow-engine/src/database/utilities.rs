use std::env;

use common::error::EmResult;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Database, PgPool, Pool, Postgres,
};

#[async_trait::async_trait]
pub trait ConnectionPool<D: Database> {
    /// Return a new pool of postgres connections
    async fn create_db_pool() -> EmResult<Pool<D>>;
    // Return a new pool of postgres connections for the test database
    async fn create_test_db_pool() -> EmResult<Pool<D>>;
}

pub struct PostgresConnectionPool;

impl PostgresConnectionPool {
    /// Return database connect options
    fn db_options() -> EmResult<PgConnectOptions> {
        let port = env::var("WE_PORT")?
            .parse()
            .expect("Port environment variable is not an integer");
        let options = PgConnectOptions::new()
            .host(&env::var("WE_HOST")?)
            .port(port)
            .database(&env::var("WE_DB")?)
            .username(&env::var("WE_USER")?)
            .password(&env::var("WE_PASSWORD")?);
        Ok(options)
    }

    /// Return test database connect options
    fn test_db_options() -> EmResult<PgConnectOptions> {
        let port = env::var("WE_TEST_PORT")?
            .parse()
            .expect("Port environment variable is not an integer");
        let options = PgConnectOptions::new()
            .host(&env::var("WE_TEST_HOST")?)
            .port(port)
            .database(&env::var("WE_TEST_DB")?)
            .username(&env::var("WE_TEST_USER")?)
            .password(&env::var("WE_TEST_PASSWORD")?);
        Ok(options)
    }
}

#[async_trait::async_trait]
impl ConnectionPool<Postgres> for PostgresConnectionPool {
    async fn create_db_pool() -> EmResult<PgPool> {
        let options = Self::db_options()?;
        let pool = PgPoolOptions::new()
            .min_connections(10)
            .max_connections(20)
            .connect_with(options)
            .await?;
        Ok(pool)
    }

    async fn create_test_db_pool() -> EmResult<PgPool> {
        let options = Self::test_db_options()?;
        let pool = PgPoolOptions::new().connect_with(options).await?;
        Ok(pool)
    }
}
