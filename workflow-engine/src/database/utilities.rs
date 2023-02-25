use std::env;

use crate::Result as WEResult;
use async_once_cell::OnceCell;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

static WE_POSTGRES_DB: OnceCell<PgPool> = OnceCell::new();

/// Return database connect options
fn db_options() -> WEResult<PgConnectOptions> {
    let port = env::var("WE_PORT")?
        .parse()
        .expect("Port environment variable is not an integer");
    let options = PgConnectOptions::new()
        .host(&env::var("WE_HOST")?)
        .port(port)
        .database(&env::var("WE_DB")?)
        .username(&env::var("WE_USER")?)
        .password(&env::var("WE_PASSWORD")?);
    Ok(options)
}

/// Return a new pool of postgres connections
pub async fn create_db_pool() -> WEResult<PgPool> {
    let options = db_options()?;
    let pool = PgPoolOptions::new()
        .min_connections(10)
        .max_connections(20)
        .connect_with(options)
        .await?;
    Ok(pool)
}

/// Get a static reference to a postgres connection pool
pub async fn we_db_pool() -> WEResult<&'static PgPool> {
    WE_POSTGRES_DB.get_or_try_init(create_db_pool()).await
}
