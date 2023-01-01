use async_once_cell::OnceCell;
use sqlx::postgres::{PgPool, PgConnectOptions, PgPoolOptions};
use std::env;

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
