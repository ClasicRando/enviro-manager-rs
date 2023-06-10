use sqlx::{
    database::HasArguments, pool::PoolConnection, types::Uuid, Connection, Database, Encode,
    Executor, IntoArguments, Pool, Transaction, Type,
};

use crate::error::{EmError, EmResult};

/// Implementors are able to provide connection pools specific to the specified [Database] type
pub trait ConnectionBuilder<D: Database> {
    /// Return a new pool of database connections. Requires the connection `options` and min/max
    /// number of connections to hold.
    async fn create_pool(
        options: <D::Connection as Connection>::Options,
        max_connections: u32,
        min_connection: u32,
    ) -> EmResult<Pool<D>>;
    /// Return a new pool of database connection with connections not explicitly created. Requires
    /// the connection `options` and min/max number of connections to hold.
    fn create_pool_lazy(
        options: <D::Connection as Connection>::Options,
        max_connections: u32,
        min_connection: u32,
    ) -> Pool<D>;
}

/// Acquire new pool connection and set the 'em.uid' parameter to the specified [Uuid]
/// # Errors
/// This function will return an error when an [Err] is returned from [Pool::acquire] or the SQL
/// call to set a config parameter.
pub async fn get_connection_with_em_uid<D>(
    uid: &Uuid,
    pool: &Pool<D>,
) -> EmResult<PoolConnection<D>>
where
    D: Database,
    for<'q> Uuid: Encode<'q, D> + Type<D>,
    for<'c> &'c mut PoolConnection<D>: Executor<'c, Database = D>,
    for<'q> <D as HasArguments<'q>>::Arguments: IntoArguments<'q, D>,
{
    let mut connection = pool.acquire().await?;
    sqlx::query("select set_config('em.uid',$1::text,false)")
        .bind(uid)
        .execute(&mut connection)
        .await?;
    Ok(connection)
}

/// Finish a transaction block by calling `COMMIT` if the `result` is [Ok] and `Rollback` if the
/// `result` is [Err]. If during the transaction `COMMIT` or `ROLLBACK` an error occurs, a
/// [EmError::CommitError] or [EmError::RollbackError] will be returned (respectively).
/// # Errors
/// This function will return an error if the original `result` is [Err] or an error is returned
/// when the transaction runs `COMMIT` or `ROLLBACK`.
pub async fn finalize_transaction<T: Send, D>(
    result: Result<T, sqlx::Error>,
    transaction: Transaction<'_, D>,
) -> EmResult<T>
where
    D: Database,
{
    match result {
        Ok(inner) => {
            if let Err(error) = transaction.commit().await {
                return Err(EmError::CommitError(error));
            }
            Ok(inner)
        }
        Err(error) => {
            if let Err(rollback_error) = transaction.rollback().await {
                return Err(EmError::RollbackError {
                    orig: error,
                    new: rollback_error,
                });
            }
            Err(error.into())
        }
    }
}
