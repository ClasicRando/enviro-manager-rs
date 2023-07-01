use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware::Logger,
    web::{get, post},
    App, HttpServer,
};
use common::error::EmResult;
use web_portal::{index, login, login_user, logout_user};

#[actix_web::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("web-portal/web_portal_log.yml", Default::default()).unwrap();
    let secret_key = Key::generate();
    let redis_connection_string = std::env::var("REDIS_CONNECTION")?;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                RedisActorSessionStore::new(&redis_connection_string),
                secret_key.clone(),
            ))
            .service(actix_files::Files::new("/assets", "web-portal/assets").show_files_listing())
            .route("/", get().to(index))
            .route("/login", get().to(login))
            .route("/logout", get().to(logout_user))
            .route("/api/login", post().to(login_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
