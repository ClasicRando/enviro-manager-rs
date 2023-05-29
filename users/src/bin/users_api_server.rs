use actix_web::cookie::Key;
use common::{database::postgres::connection::PgConnectionBuilder, error::EmResult};
use users::{
    api,
    database::db_options,
    service::postgres::{roles::PgRoleService, users::PgUserService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("users/users_api_server_log.yml", Default::default()).unwrap();
    let signing_key = Key::generate();
    api::spawn_api_server::<
        (&str, u16),
        PgConnectionBuilder,
        _,
        PgRoleService,
        PgUserService,
    >(("127.0.0.1", 8080), db_options()?, signing_key)
    .await?;
    Ok(())
}
