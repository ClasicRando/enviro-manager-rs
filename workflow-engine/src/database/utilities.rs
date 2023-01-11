use std::env;

use async_once_cell::OnceCell;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Postgres, Transaction,
};

use crate::error::{Error as WEError, Result as WEResult};

static WE_POSTGRES_DB: OnceCell<PgPool> = OnceCell::new();

/// Return database connect options
fn db_options() -> PgConnectOptions {
    let port = env!("WE_PORT")
        .parse()
        .expect("Port environment variable is not an integer");
    PgConnectOptions::new()
        .host(env!("WE_HOST"))
        .port(port)
        .database(env!("WE_DB"))
        .username(env!("WE_USER"))
        .password(env!("WE_PASSWORD"))
}

/// Return a new pool of postgres connections
pub async fn create_db_pool() -> Result<PgPool, sqlx::Error> {
    let options = db_options();
    let pool = PgPoolOptions::new()
        .min_connections(10)
        .max_connections(20)
        .connect_with(options)
        .await?;
    Ok(pool)
}

/// Get a static reference to a postgres connection pool
pub async fn we_db_pool() -> Result<&'static PgPool, sqlx::Error> {
    WE_POSTGRES_DB.get_or_try_init(create_db_pool()).await
}

/// Complete a transaction depending upon the result of a sql operation.
///
/// If the result is [Ok] then [`commit_transaction`] is called. If the result is [Err] then
/// [`rollback_transaction`] is called.
pub async fn finish_transaction<T>(
    transaction: Transaction<'_, Postgres>,
    result: Result<T, sqlx::Error>,
) -> WEResult<T> {
    match result {
        Ok(inner) => commit_transaction(inner, transaction).await,
        Err(error) => rollback_transaction(error, transaction).await,
    }
}

/// Attempts to commit the transaction, returning a [`CommitError`][WEError::CommitError] if that
/// fails. Otherwise, returns the inner value.
pub async fn commit_transaction<T>(
    inner: T,
    transaction: Transaction<'_, Postgres>,
) -> WEResult<T> {
    if let Err(error) = transaction.commit().await {
        return Err(WEError::CommitError(error));
    }
    Ok(inner)
}

/// Attempts to rollback the transaction, returning a [`RollbackError`][WEError::RollbackError] if
/// that fails (both errors are retained). Otherwise, returns the original error.
pub async fn rollback_transaction<T>(
    error: sqlx::Error,
    transaction: Transaction<'_, Postgres>,
) -> WEResult<T> {
    match transaction.rollback().await {
        Ok(_) => Err(WEError::Sql(error)),
        Err(t_error) => Err(WEError::RollbackError {
            orig: error,
            new: t_error,
        }),
    }
}
