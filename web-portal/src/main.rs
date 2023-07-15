use cfg_if::cfg_if;

// boilerplate to run in different modes
// cfg_if! {}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> common::error::EmResult<()> {
    use actix_files::Files;
    use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
    use actix_web::{cookie::Key, *};
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use web_portal::app::*;

    if let Err(error) = log4rs::init_file("web-portal/web_portal_log.yml", Default::default()) {
        println!("Could not start logging. {error}");
        return Ok(());
    };

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    let key_env = std::env::var("SECRET_KEY")?;
    let secret_key = Key::from(key_env.as_bytes());
    let redis_connection_string = std::env::var("REDIS_CONNECTION")?;

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(
                    RedisActorSessionStore::new(&redis_connection_string),
                    secret_key.clone(),
                )
                .cookie_http_only(true)
                // .cookie_path("/".to_owned())
                .cookie_same_site(cookie::SameSite::Strict)
                .build(),
            )
            .wrap(middleware::Compress::default())
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .service(Files::new("/", site_root))
    })
    .bind(&addr)?
    .run()
    .await?;
    Ok(())
}

#[cfg(not(any(feature = "ssr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `ssg` instead
}
