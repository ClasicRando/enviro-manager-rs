use std::net::ToSocketAddrs;

use actix_web::{
    web::{delete, get, patch, post, Data},
    App, HttpServer,
};
use common::{database::ConnectionPool, error::EmResult};
use sqlx::{Connection, Database};

pub mod roles;
pub mod users;

use crate::services::{
    create_roles_service, create_users_service, roles::RoleService, users::UserService,
};

pub async fn spawn_api_server<A, C, D, R, U>(
    address: A,
    options: <D::Connection as Connection>::Options,
) -> EmResult<()>
where
    A: ToSocketAddrs,
    C: ConnectionPool<D>,
    D: Database,
    R: RoleService<Database = D> + Send + Sync + 'static,
    U: UserService<Database = D> + Send + Sync + 'static,
{
    let pool = C::create_db_pool(options).await?;
    let roles_service: Data<R> = Data::new(create_roles_service::<R, D>(&pool)?);
    let users_service: Data<U> = Data::new(create_users_service::<U, D>(&pool)?);
    HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .app_data(roles_service.clone())
                .app_data(users_service.clone())
                .route("/roles", get().to(roles::roles::<R>))
                .route("/roles", post().to(roles::create_role::<R>))
                .route("/roles", patch().to(roles::update_role::<R>))
                .route("/users", post().to(users::create_user::<U>))
                .route("/users/username", patch().to(users::update_username::<U>))
                .route("/users/full-name", patch().to(users::update_full_name::<U>))
                .route("/users/validate", patch().to(users::validate_user::<U>))
                .route(
                    "/users/reset-password",
                    patch().to(users::reset_password::<U>),
                )
                .route("/users/role", post().to(users::add_user_role::<U>))
                .route("/users/role", delete().to(users::revoke_user_role::<U>)),
        )
    })
    .bind(address)?
    .run()
    .await?;
    Ok(())
}
