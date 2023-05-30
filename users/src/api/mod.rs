use std::net::ToSocketAddrs;

use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    middleware::Logger,
    web::{get, patch, post, Data},
    App, HttpServer,
};
use common::{database::Database, error::EmResult};

pub mod roles;
pub mod users;

use crate::service::{roles::RoleService, users::UserService};

/// Run generic API server. Creates all the required endpoints and resources. To run the api server,
/// you must have created a [ConnectionBuilder], [RoleService] and [UserService] for your desired
/// [Database] implementation. Each component depends of a [Database] type so the system cannot
/// contain disjointed service implementations to operate.
pub async fn spawn_api_server<A, D, R, U>(
    address: A,
    options: D::ConnectionOptions,
    signing_key: Key,
) -> EmResult<()>
where
    A: ToSocketAddrs,
    D: Database,
    R: RoleService<UserService = U> + Send + Sync + 'static,
    U: UserService<Database = D> + Send + Sync + 'static,
{
    let pool = D::create_pool(options, 20, 10).await?;
    let users_service = U::create(&pool);
    let roles_service_data: Data<R> = Data::new(R::create(&users_service));
    let users_service_data: Data<U> = Data::new(users_service);
    let redis_connection_string = std::env::var("REDIS_CONNECTION")?;
    HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .wrap(Logger::default())
                .wrap(
                    SessionMiddleware::builder(
                        RedisActorSessionStore::new(&redis_connection_string),
                        signing_key.clone(),
                    )
                    .cookie_http_only(false)
                    .cookie_same_site(SameSite::Strict)
                    .build(),
                )
                .app_data(roles_service_data.clone())
                .app_data(users_service_data.clone())
                .route("/roles", get().to(roles::roles::<R>))
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
