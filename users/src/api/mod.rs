use std::net::ToSocketAddrs;

use actix_web::{
    middleware::Logger,
    web::{get, patch, post, Data},
    App, HttpServer,
};
use common::{database::Database, error::EmResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod roles;
pub mod users;

use crate::service::{roles::RoleService, users::UserService};

/// Simple request with only the users uuid in the body
#[derive(Serialize, Deserialize, Debug)]
pub struct UidRequest {
    /// uuid of the user attempting to perform the action
    pub(crate) uid: Uuid,
}

/// Run generic API server. Creates all the required endpoints and resources. To run the api server,
/// you must have created a [ConnectionBuilder], [RoleService] and [UserService] for your desired
/// [Database] implementation. Each component depends of a [Database] type so the system cannot
/// contain disjointed service implementations to operate.
/// # Errors
/// This function will return an error if the server is unable to bind to the specified `address` or
/// the server's `run` method returns an error
pub async fn spawn_api_server<A, D, R, U>(
    users_service: U,
    roles_service: R,
    address: A,
) -> EmResult<()>
where
    A: ToSocketAddrs,
    D: Database,
    R: RoleService<UserService = U> + Send + Sync + 'static,
    U: UserService<Database = D> + Send + Sync + 'static,
{
    let roles_service_data: Data<R> = Data::new(roles_service);
    let users_service_data: Data<U> = Data::new(users_service);
    HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .wrap(Logger::default())
                .app_data(roles_service_data.clone())
                .app_data(users_service_data.clone())
                .route("/roles", get().to(roles::roles::<R>))
                .route("/users", get().to(users::read_users::<U>))
                .route("/users", post().to(users::create_user::<U>))
                .route("/users", patch().to(users::update_user::<U>))
                .route("/users/validate", post().to(users::validate_user::<U>))
                .route("/users/role", post().to(users::modify_user_role::<U>)),
        )
    })
    .bind(address)?
    .run()
    .await?;
    Ok(())
}
