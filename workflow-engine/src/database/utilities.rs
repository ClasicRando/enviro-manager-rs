use async_once_cell::OnceCell;
use sqlx::{
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    Postgres, Transaction,
};
use std::{env, fmt::Display};

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

#[derive(Debug)]
pub enum TransactionError {
    Sql(sqlx::Error),
    CommitError(sqlx::Error),
    RollbackError { orig: sqlx::Error, new: sqlx::Error },
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::Sql(e) => write!(f, "{}", e),
            TransactionError::CommitError(e) => write!(f, "Error during transaction commit: {}", e),
            TransactionError::RollbackError { orig, new } => write!(
                f,
                "Error during transaction rollback: {}\nOriginal Error: {}",
                new, orig
            ),
        }
    }
}

pub async fn finish_transaction<T>(
    transaction: Transaction<'_, Postgres>,
    result: Result<T, sqlx::Error>,
) -> Result<T, TransactionError> {
    match result {
        Ok(inner) => {
            if let Err(error) = transaction.commit().await {
                return Err(TransactionError::CommitError(error));
            }
            Ok(inner)
        }
        Err(error) => match transaction.rollback().await {
            Ok(_) => Err(TransactionError::Sql(error)),
            Err(t_error) => Err(TransactionError::RollbackError {
                orig: error,
                new: t_error,
            }),
        },
    }
}
