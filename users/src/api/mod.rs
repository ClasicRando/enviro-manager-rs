use std::net::ToSocketAddrs;

use actix_web::{
    web::{get, patch, post, Data},
    App, HttpServer,
};
use common::{database::ConnectionBuilder, error::EmResult};
use sqlx::{Connection, Database};

pub mod roles;
pub mod users;

use crate::service::{roles::RoleService, users::UserService};

pub async fn spawn_api_server<A, C, D, R, U>(
    address: A,
    options: <D::Connection as Connection>::Options,
) -> EmResult<()>
where
    A: ToSocketAddrs,
    C: ConnectionBuilder<D>,
    D: Database,
    R: RoleService<Database = D> + Send + Sync + 'static,
    U: UserService<Database = D> + Send + Sync + 'static,
{
    let pool = C::create_pool(options, 20, 10).await?;
    let roles_service: Data<R> = Data::new(R::new(&pool));
    let users_service: Data<U> = Data::new(U::new(&pool));
    HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .app_data(roles_service.clone())
                .app_data(users_service.clone())
                .route("/roles", get().to(roles::roles::<R>))
                .route("/roles", post().to(roles::create_role::<R>))
                .route("/roles", patch().to(roles::update_role::<R>))
                .route("/users", post().to(users::create_user::<U>))
                .route("/users/username", patch().to(users::update_user::<U>))
                .route("/users/validate", patch().to(users::validate_user::<U>))
                .route("/users/role", post().to(users::modify_user_role::<U>)),
        )
    })
    .bind(address)?
    .run()
    .await?;
    Ok(())
}
