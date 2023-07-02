use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware::Logger,
    web::{get, post},
    App, HttpResponse, HttpServer,
};
use common::error::EmResult;
use web_portal::{
    api::{login_user, logout_user},
    pages::{index, login},
};

async fn redirect_home() -> HttpResponse {
    HttpResponse::Found()
        .insert_header(("location", "/"))
        .finish()
}

#[actix_web::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("web-portal/web_portal_log.yml", Default::default()).unwrap();
    let secret = std::env::var("SECRET_KEY")?;
    let secret_key = Key::from(secret.as_bytes());
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
            .route("/index", get().to(redirect_home))
            .route("/login", get().to(login))
            .route("/logout", get().to(logout_user))
            .route("/api/login", post().to(login_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
