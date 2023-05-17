use std::env;

use common::error::EmResult;
use sqlx::postgres::PgConnectOptions;

/// Return database connect options
pub fn db_options() -> EmResult<PgConnectOptions> {
    let port = env::var("USERS_PORT")?
        .parse()
        .expect("Port environment variable is not an integer");
    let options: PgConnectOptions = PgConnectOptions::new()
        .host(&env::var("USERS_HOST")?)
        .port(port)
        .database(&env::var("USERS_DB")?)
        .username(&env::var("USERS_USER")?)
        .password(&env::var("USERS_PASSWORD")?);
    Ok(options)
}

/// Return database connect options for web user
pub fn db_web_options() -> EmResult<PgConnectOptions> {
    let port = env::var("USERS_WEB_PORT")?
        .parse()
        .expect("Port environment variable is not an integer");
    let options: PgConnectOptions = PgConnectOptions::new()
        .host(&env::var("USERS_WEB_HOST")?)
        .port(port)
        .database(&env::var("USERS_WEB_DB")?)
        .username(&env::var("USERS_WEB_USER")?)
        .password(&env::var("USERS_WEB_PASSWORD")?);
    Ok(options)
}

/// Return test database connect options
pub fn test_db_options() -> EmResult<PgConnectOptions> {
    let port = env::var("USERS_TEST_PORT")?
        .parse()
        .expect("Port environment variable is not an integer");
    let options = PgConnectOptions::new()
        .host(&env::var("USERS_TEST_HOST")?)
        .port(port)
        .database(&env::var("USERS_TEST_DB")?)
        .username(&env::var("USERS_TEST_USER")?)
        .password(&env::var("USERS_TEST_PASSWORD")?);
    Ok(options)
}
