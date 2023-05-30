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

#[cfg(test)]
mod test {
    use common::database::{
        connection::ConnectionBuilder, postgres::connection::PgConnectionBuilder,
        test::run_db_tests,
    };

    use crate::database::db_options;

    #[tokio::test]
    async fn run_workflow_engine_database_tests() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        run_db_tests(&pool).await?;
        Ok(())
    }
}
