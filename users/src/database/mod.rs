use std::env;

use common::error::EmResult;
use sqlx::postgres::PgConnectOptions;

/// Return database connect options
/// # Errors
/// This function returns an error if any of the required environment variables are not present or
/// the port environment variable cannot be parsed as a [u16]. The environment variables required
/// are:
/// - USERS_HOST -> address to the postgres database server
/// - USERS_PORT -> port that the postgres database is listening
/// - USERS_DB -> name of the database to connect
/// - USERS_USER -> name of the user to connect as
/// - USERS_PASSWORD -> password of the user to connect as
pub fn db_options() -> EmResult<PgConnectOptions> {
    let port = env::var("USERS_PORT")?.parse()?;
    let options: PgConnectOptions = PgConnectOptions::new()
        .host(&env::var("USERS_HOST")?)
        .port(port)
        .database(&env::var("USERS_DB")?)
        .username(&env::var("USERS_USER")?)
        .password(&env::var("USERS_PASSWORD")?);
    Ok(options)
}
