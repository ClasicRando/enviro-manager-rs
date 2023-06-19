#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> common::error::EmResult<()> {
    use actix_files::Files;
    use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
    use actix_web::{cookie::Key, *};
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use web_portal::{
        api::user::{GetUser, LoginUser},
        app::*,
    };

    _ = LoginUser::register();
    _ = GetUser::register();

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    let secret_key = Key::generate();
    let redis_connection_string = std::env::var("REDIS_CONNECTION")?;

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .wrap(SessionMiddleware::new(
                RedisActorSessionStore::new(&redis_connection_string),
                secret_key.clone(),
            ))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .service(Files::new("/", site_root))
        //.wrap(middleware::Compress::default())
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
