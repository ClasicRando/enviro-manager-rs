use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web::get, App, HttpServer};
use common::error::EmResult;
use web_portal::login;

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
            .route("/", get().to(login))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
