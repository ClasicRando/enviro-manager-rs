use common::{
    database::{postgres::Postgres, Database},
    error::EmResult,
};
use users::{
    api,
    database::db_options,
    service::postgres::{roles::PgRoleService, users::PgUserService},
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("users/users_api_server_log.yml", Default::default()).unwrap();
    let options = db_options()?;
    let pool = Postgres::create_pool(options, 20, 10).await?;
    let users_service = PgUserService::new(&pool);
    let roles_service = PgRoleService::new(&users_service);
    api::spawn_api_server(users_service, roles_service, ("127.0.0.1", 8080)).await?;
    Ok(())
}
