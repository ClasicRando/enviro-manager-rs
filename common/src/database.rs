use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Database, PgPool, Pool, Postgres,
};

use crate::error::EmResult;

#[async_trait::async_trait]
pub trait ConnectionPool<D: Database> {
    /// Return a new pool of postgres connections
    async fn create_db_pool(options: <D::Connection as Connection>::Options) -> EmResult<Pool<D>>;
    // Return a new pool of postgres connections for the test database
    async fn create_test_db_pool(
        options: <D::Connection as Connection>::Options,
    ) -> EmResult<Pool<D>>;
}

pub struct PgConnectionPool;

#[async_trait::async_trait]
impl ConnectionPool<Postgres> for PgConnectionPool {
    async fn create_db_pool(options: PgConnectOptions) -> EmResult<PgPool> {
        let pool = PgPoolOptions::new()
            .min_connections(10)
            .max_connections(20)
            .connect_with(options)
            .await?;
        Ok(pool)
    }

    async fn create_test_db_pool(options: PgConnectOptions) -> EmResult<PgPool> {
        let pool = PgPoolOptions::new().connect_with(options).await?;
        Ok(pool)
    }
}
