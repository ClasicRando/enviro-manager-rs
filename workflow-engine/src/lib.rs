use sqlx::{postgres::{PgConnectOptions, PgPoolOptions}, PgPool};

fn we_db_options() -> Result<PgConnectOptions, sqlx::Error> {
    let we_host_address = env!("WE_HOST");
    let we_db_name = env!("WE_DB");
    let we_db_user = env!("WE_USER");
    let we_db_password = env!("WE_PASSWORD");
    let options = PgConnectOptions::new()
        .host(we_host_address)
        .database(we_db_name)
        .options([("search_path", "workflow_engine")])
        .username(we_db_user)
        .password(we_db_password);
    Ok(options)
}

pub async fn create_we_db_pool() -> Result<PgPool, sqlx::Error> {
    let options = we_db_options()?;
    let pool = PgPoolOptions::new()
        .min_connections(10)
        .max_connections(20)
        .connect_with(options)
        .await?;
    Ok(pool)
}
