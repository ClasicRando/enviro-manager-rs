use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Postgres,
};

use crate::{database::connection::ConnectionBuilder, error::EmResult};

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

    fn create_pool_lazy(
        options: PgConnectOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> PgPool {
        PgPoolOptions::new()
            .min_connections(min_connection)
            .max_connections(max_connections)
            .connect_lazy_with(options)
    }
}
