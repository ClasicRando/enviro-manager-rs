use std::env;

use common::error::EmResult;
use sqlx::postgres::PgConnectOptions;

/// Return database connect options
pub fn db_options() -> EmResult<PgConnectOptions> {
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
