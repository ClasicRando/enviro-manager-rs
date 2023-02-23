use std::env;

use async_once_cell::OnceCell;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

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
