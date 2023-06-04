use std::env;

use common::error::EmResult;
use sqlx::postgres::PgConnectOptions;

/// Return database connect options
/// # Errors
/// This function returns an error if any of the required environment variables are not present or
/// the port environment variable cannot be parsed as a [u16]. The environment variables required
/// are:
/// - WE_HOST -> address to the postgres database server
/// - WE_PORT -> port that the postgres database is listening
/// - WE_DB -> name of the database to connect
/// - WE_USER -> name of the user to connect as
/// - WE_PASSWORD -> password of the user to connect as
pub fn db_options() -> EmResult<PgConnectOptions> {
    let port = env::var("WE_PORT")?.parse()?;
    let options = PgConnectOptions::new()
        .host(&env::var("WE_HOST")?)
        .port(port)
        .database(&env::var("WE_DB")?)
        .username(&env::var("WE_USER")?)
        .password(&env::var("WE_PASSWORD")?);
    Ok(options)
}
