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

#[cfg(test)]
mod test {
    use common::{
        database::{ConnectionBuilder, PgConnectionBuilder},
        db_test::run_db_tests,
    };

    use super::test_db_options;

    #[tokio::test]
    async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgConnectionBuilder::create_pool(test_db_options()?, 1, 1).await?;
        run_db_tests(&pool).await?;
        Ok(())
    }
}
