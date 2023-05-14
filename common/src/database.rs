use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Database, PgPool, Pool, Postgres,
};

use crate::error::EmResult;

pub trait ConnectionBuilder<D: Database> {
    /// Return a new pool of database connections. Requires the connection `options` and min/max
    /// number of connections to hold.
    async fn create_pool(
        options: <D::Connection as Connection>::Options,
        max_connections: u32,
        min_connection: u32,
    ) -> EmResult<Pool<D>>;
}

pub struct PgConnectionBuilder;

impl ConnectionBuilder<Postgres> for PgConnectionBuilder {
    async fn create_pool(
        options: PgConnectOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> EmResult<PgPool> {
        let pool = PgPoolOptions::new()
            .min_connections(min_connection)
            .max_connections(max_connections)
            .connect_with(options)
            .await?;
        Ok(pool)
    }
}
