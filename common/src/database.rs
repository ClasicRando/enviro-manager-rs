use sqlx::{
    database::HasArguments,
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgPoolOptions},
    types::Uuid,
    Connection, Database, Encode, Executor, IntoArguments, PgPool, Pool, Postgres, Type,
};

use crate::error::EmResult;

/// Implementors are able to provide connection pools specific to the specified [Database] type
pub trait ConnectionBuilder<D: Database> {
    /// Return a new pool of database connections. Requires the connection `options` and min/max
    /// number of connections to hold.
    async fn create_pool(
        options: <D::Connection as Connection>::Options,
        max_connections: u32,
        min_connection: u32,
    ) -> EmResult<Pool<D>>;
    /// Return a new pool of database connection with connections not explicitly created. Requires the
    /// connection `options` and min/max number of connections to hold.
    fn create_pool_lazy(
        options: PgConnectOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> PgPool;
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

/// Acquire new pool connection and set the 'em.uid' parameter to the specified [Uuid]
pub async fn get_connection_with_em_uid<D>(em_uid: Uuid, pool: &Pool<D>) -> EmResult<PoolConnection<D>>
where
    D: Database,
    for<'q> Uuid: Encode<'q, D> + Type<D>,
    for<'c> &'c mut PoolConnection<D>: Executor<'c, Database = D>,
    for<'q> <D as HasArguments<'q>>::Arguments: IntoArguments<'q, D>,
{
    let mut connection = pool.acquire().await?;
    sqlx::query("select set_config('em.uid',$1::text,false)")
        .bind(em_uid)
        .execute(&mut connection)
        .await?;
    Ok(connection)
}
