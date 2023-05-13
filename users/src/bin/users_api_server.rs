use common::{database::PgConnectionPool, error::EmResult};
use sqlx::Postgres;
use users::{
    api,
    database::db_options,
    service::{roles::PgRoleService, users::PgUserService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("users/api_server_log.yml", Default::default()).unwrap();
    api::spawn_api_server::<(&str, u16), PgConnectionPool, Postgres, PgRoleService, PgUserService>(
        ("127.0.0.1", 8080),
        db_options()?,
    )
    .await?;
    Ok(())
}
