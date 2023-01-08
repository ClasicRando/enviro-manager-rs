use std::env;

use async_once_cell::OnceCell;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Postgres, Transaction,
};

use crate::error::{Error as WEError, Result as WEResult};

static WE_POSTGRES_DB: OnceCell<PgPool> = OnceCell::new();

fn db_options() -> Result<PgConnectOptions, sqlx::Error> {
    let options = PgConnectOptions::new()
        .host(env!("WE_HOST"))
        .port(9875)
        .database(env!("WE_DB"))
        .username(env!("WE_USER"))
        .password(env!("WE_PASSWORD"));
    Ok(options)
}

pub async fn create_db_pool() -> Result<PgPool, sqlx::Error> {
    let options = db_options()?;
    let pool = PgPoolOptions::new()
        .min_connections(10)
        .max_connections(20)
        .connect_with(options)
        .await?;
    Ok(pool)
}

pub async fn we_db_pool() -> Result<&'static PgPool, sqlx::Error> {
    WE_POSTGRES_DB.get_or_try_init(create_db_pool()).await
}

pub async fn finish_transaction<T>(
    transaction: Transaction<'_, Postgres>,
    result: Result<T, sqlx::Error>,
) -> WEResult<T> {
    match result {
        Ok(inner) => commit_transaction(inner, transaction).await,
        Err(error) => rollback_transaction(error, transaction).await,
    }
}

pub async fn commit_transaction<T>(
    inner: T,
    transaction: Transaction<'_, Postgres>,
) -> WEResult<T> {
    if let Err(error) = transaction.commit().await {
        return Err(WEError::CommitError(error));
    }
    Ok(inner)
}

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
