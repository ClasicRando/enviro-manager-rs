use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use common::{database::connection::PgConnectionBuilder, error::EmResult};
use sqlx::Postgres;
use common::error::EmError;
use users::{
    api,
    database::db_web_options,
    service::postgres::{roles::PgRoleService, users::PgUserService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("users/users_api_server_log.yml", Default::default()).unwrap();
    let redis_connection_string = std::env::var("REDIS_CONNECTION")?;
    let redis_session = RedisSessionStore::new(&redis_connection_string)
        .await
        .map_err(|err| EmError::Generic(format!("Error: {err}")))?;
    let signing_key = Key::generate();
    api::spawn_api_server::<
        (&str, u16),
        PgConnectionBuilder,
        Postgres,
        PgRoleService,
        RedisSessionStore,
        PgUserService,
    >(("127.0.0.1", 8080), db_web_options()?, redis_session, signing_key)
    .await?;
    Ok(())
}
