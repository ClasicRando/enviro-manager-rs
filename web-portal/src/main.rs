use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware::Logger,
    web::{get, post},
    App, HttpResponse, HttpServer,
};
use common::error::EmResult;
use web_portal::{
    api::{
        active_executors, active_executors_html, active_workflow_runs, active_workflow_runs_html,
        login_user, logout_user,
    },
    pages::{index, login, workflow_engine},
};

async fn redirect_home() -> HttpResponse {
    web_portal::utils::redirect_home!()
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
            .route("/workflow-engine", get().to(workflow_engine))
            .route("/api/login", post().to(login_user))
            .route("/api/workflow-engine/executors", get().to(active_executors))
            .route(
                "/api/html/workflow-engine/executors",
                get().to(active_executors_html),
            )
            .route(
                "/api/workflow-engine/workflow-runs",
                get().to(active_workflow_runs),
            )
            .route(
                "/api/html/workflow-engine/workflow-runs",
                get().to(active_workflow_runs_html),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
