use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use crate::{database::Database, error::EmResult};

pub mod build;
pub mod connection;
pub mod listener;

/// Postgresql implementation of the [Database] interface
pub struct Postgres;

impl Database for Postgres {
    type ConnectionOptions = PgConnectOptions;
    type ConnectionPool = PgPool;

    async fn create_pool(
        options: Self::ConnectionOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> EmResult<Self::ConnectionPool> {
        let pool = PgPoolOptions::new()
            .min_connections(min_connection)
            .max_connections(max_connections)
            .connect_with(options)
            .await?;
        Ok(pool)
    }

    fn create_pool_lazy(
        options: Self::ConnectionOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> Self::ConnectionPool {
        PgPoolOptions::new()
            .min_connections(min_connection)
            .max_connections(max_connections)
            .connect_lazy_with(options)
    }
}
