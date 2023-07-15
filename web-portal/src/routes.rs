use std::{fmt::Display, future::Future, sync::Arc};

use actix_web::{
    body::BoxBody,
    dev::{ServiceFactory, ServiceRequest},
    http::header,
    web::Bytes,
    *,
};
use futures::{Stream, StreamExt};
use http::StatusCode;
use leptos::{
    leptos_server::{server_fn_by_path, Payload},
    server_fn::Encoding,
    ssr::render_to_stream_with_prefix_undisposed_with_context_and_block_replacement,
    *,
};
use leptos_actix::{ResponseOptions, ResponseParts};
use leptos_integration_utils::{build_async_response, html_parts_separated};
use leptos_meta::*;
use leptos_router::*;

/// An Actix [Route](actix_web::Route) that listens for a `POST` request with
/// Leptos server function arguments in the body, runs the server function if found,
/// and returns the resulting [HttpResponse].
///
/// This provides the [HttpRequest] to the server [Scope](leptos::Scope).
///
/// This can then be set up at an appropriate route in your application:
///
/// ```
/// use actix_web::*;
///
/// fn register_server_functions() {
///     // call ServerFn::register() for each of the server functions you've defined
/// }
///
/// # if false { // don't actually try to run a server in a doctest...
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     // make sure you actually register your server functions
///     register_server_functions();
///
///     HttpServer::new(|| {
///         App::new()
///             // "/api" should match the prefix, if any, declared when defining server functions
///             // {tail:.*} passes the remainder of the URL as the server function name
///             .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
///     })
///     .bind(("127.0.0.1", 8080))?
///     .run()
///     .await
/// }
/// # }
/// ```
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
pub fn handle_server_fns() -> Route {
    handle_server_fns_with_context(|_cx| {})
}

/// An Actix [Route](actix_web::Route) that listens for `GET` or `POST` requests with
/// Leptos server function arguments in the URL (`GET`) or body (`POST`),
/// runs the server function if found, and returns the resulting [HttpResponse].
///
/// This provides the [HttpRequest] to the server [Scope](leptos::Scope).
///
/// This can then be set up at an appropriate route in your application:
///
/// This version allows you to pass in a closure that adds additional route data to the
/// context, allowing you to pass in info about the route or user from Actix, or other info.
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
pub fn handle_server_fns_with_context(
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
) -> Route {
    web::to(
        move |req: HttpRequest, params: web::Path<String>, body: web::Bytes| {
            let additional_context = additional_context.clone();
            async move {
                let additional_context = additional_context.clone();

                let path = params.into_inner();
                let accept_header = req
                    .headers()
                    .get("Accept")
                    .and_then(|value| value.to_str().ok());

                if let Some(server_fn) = server_fn_by_path(path.as_str()) {
                    let body_ref: &[u8] = &body;

                    let runtime = create_runtime();
                    let (cx, disposer) = raw_scope_and_disposer(runtime);

                    // Add additional info to the context of the server function
                    additional_context(cx);
                    let res_options = ResponseOptions::default();

                    // provide HttpRequest as context in server scope
                    provide_context(cx, req.clone());
                    provide_context(cx, res_options.clone());

                    // we consume the body here (using the web::Bytes extractor), but it is required
                    // for things like MultipartForm
                    if req
                        .headers()
                        .get("Content-Type")
                        .and_then(|value| value.to_str().ok())
                        .map(|value| value.starts_with("multipart/form-data; boundary="))
                        == Some(true)
                    {
                        provide_context(cx, body.clone());
                    }

                    let query = req.query_string().as_bytes();

                    let data = match &server_fn.encoding() {
                        Encoding::Url | Encoding::Cbor => body_ref,
                        Encoding::GetJSON | Encoding::GetCBOR => query,
                    };
                    let res = match server_fn.call(cx, data).await {
                        Ok(serialized) => {
                            let res_options = use_context::<ResponseOptions>(cx).unwrap();

                            let mut res: HttpResponseBuilder = HttpResponse::Ok();
                            let res_parts = res_options.0.write();

                            // if accept_header isn't set to one of these, it's a form submit
                            // redirect back to the referrer if not redirect has been set
                            if accept_header != Some("application/json")
                                && accept_header != Some("application/x-www-form-urlencoded")
                                && accept_header != Some("application/cbor")
                            {
                                // Location will already be set if redirect() has been used
                                let has_location_set = res_parts.headers.get("Location").is_some();
                                if !has_location_set {
                                    let referer = req
                                        .headers()
                                        .get("Referer")
                                        .and_then(|value| value.to_str().ok())
                                        .unwrap_or("/");
                                    res = HttpResponse::SeeOther();
                                    res.insert_header(("Location", referer))
                                        .content_type("application/json");
                                }
                            };
                            // Override StatusCode if it was set in a Resource or Element
                            if let Some(status) = res_parts.status {
                                res.status(status);
                            }

                            // Use provided ResponseParts headers if they exist
                            let _count = res_parts
                                .headers
                                .clone()
                                .into_iter()
                                .map(|(k, v)| {
                                    res.append_header((k, v));
                                })
                                .count();

                            match serialized {
                                Payload::Binary(data) => {
                                    res.content_type("application/cbor");
                                    res.body(Bytes::from(data))
                                }
                                Payload::Url(data) => {
                                    res.content_type("application/x-www-form-urlencoded");
                                    res.body(data)
                                }
                                Payload::Json(data) => {
                                    res.content_type("application/json");
                                    res.body(data)
                                }
                            }
                        }
                        Err(e) => HttpResponse::InternalServerError()
                            .body(serde_json::to_string(&e).unwrap_or_else(|_| e.to_string())),
                    };
                    // clean up the scope
                    disposer.dispose();
                    runtime.dispose();
                    res
                } else {
                    HttpResponse::BadRequest().body(format!(
                        "Could not find a server function at the route {:?}. \n\nIt's likely that \
                         you need to call ServerFn::register_explicit() on the server function \
                         type, somewhere in your `main` function.",
                        req.path()
                    ))
                }
            }
        },
    )
}

/// Returns an Actix [Route](actix_web::Route) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application. The stream
/// will include fallback content for any `<Suspense/>` nodes, and be immediately interactive,
/// but requires some client-side JavaScript.
///
/// The provides a [MetaContext] and a [RouterIntegrationContext] to app’s context before
/// rendering it, and includes any meta tags injected using [leptos_meta].
///
/// The HTML stream is rendered using [render_to_stream](leptos::ssr::render_to_stream), and
/// includes everything described in the documentation for that function.
///
/// This can then be set up at an appropriate route in your application:
/// ```
/// use std::{env, net::SocketAddr};
///
/// use actix_web::{App, HttpServer};
/// use leptos::*;
/// use leptos_router::Method;
///
/// #[component]
/// fn MyApp(cx: Scope) -> impl IntoView {
///     view! { cx, <main>"Hello, world!"</main> }
/// }
///
/// # if false { // don't actually try to run a server in a doctest...
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
///     let addr = conf.leptos_options.site_addr.clone();
///     HttpServer::new(move || {
///         let leptos_options = &conf.leptos_options;
///
///         App::new()
///             // {tail:.*} passes the remainder of the URL as the route
///             // the actual routing will be handled by `leptos_router`
///             .route(
///                 "/{tail:.*}",
///                 leptos_actix::render_app_to_stream(
///                     leptos_options.to_owned(),
///                     |cx| view! { cx, <MyApp/> },
///                     Method::Get,
///                 ),
///             )
///     })
///     .bind(&addr)?
///     .run()
///     .await
/// }
/// # }
/// ```
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
/// - [MetaContext](leptos_meta::MetaContext)
/// - [RouterIntegrationContext](leptos_router::RouterIntegrationContext)
pub fn render_app_to_stream<IV>(
    options: LeptosOptions,
    app_fn: impl Fn(leptos::Scope) -> IV + Clone + 'static,
    method: Method,
) -> Route
where
    IV: IntoView,
{
    render_app_to_stream_with_context(options, |_cx| {}, app_fn, method)
}

/// Returns an Actix [Route](actix_web::Route) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an in-order HTML stream of your application.
/// This stream will pause at each `<Suspense/>` node and wait for it to resolve before
/// sending down its HTML. The app will become interactive once it has fully loaded.
///
/// The provides a [MetaContext] and a [RouterIntegrationContext] to app’s context before
/// rendering it, and includes any meta tags injected using [leptos_meta].
///
/// The HTML stream is rendered using
/// [render_to_stream_in_order](leptos::ssr::render_to_stream_in_order),
/// and includes everything described in the documentation for that function.
///
/// This can then be set up at an appropriate route in your application:
/// ```
/// use std::{env, net::SocketAddr};
///
/// use actix_web::{App, HttpServer};
/// use leptos::*;
/// use leptos_router::Method;
///
/// #[component]
/// fn MyApp(cx: Scope) -> impl IntoView {
///     view! { cx, <main>"Hello, world!"</main> }
/// }
///
/// # if false { // don't actually try to run a server in a doctest...
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
///     let addr = conf.leptos_options.site_addr.clone();
///     HttpServer::new(move || {
///         let leptos_options = &conf.leptos_options;
///
///         App::new()
///             // {tail:.*} passes the remainder of the URL as the route
///             // the actual routing will be handled by `leptos_router`
///             .route(
///                 "/{tail:.*}",
///                 leptos_actix::render_app_to_stream_in_order(
///                     leptos_options.to_owned(),
///                     |cx| view! { cx, <MyApp/> },
///                     Method::Get,
///                 ),
///             )
///     })
///     .bind(&addr)?
///     .run()
///     .await
/// }
/// # }
/// ```
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
/// - [MetaContext](leptos_meta::MetaContext)
/// - [RouterIntegrationContext](leptos_router::RouterIntegrationContext)
pub fn render_app_to_stream_in_order<IV>(
    options: LeptosOptions,
    app_fn: impl Fn(leptos::Scope) -> IV + Clone + 'static,
    method: Method,
) -> Route
where
    IV: IntoView,
{
    render_app_to_stream_in_order_with_context(options, |_cx| {}, app_fn, method)
}

/// Returns an Actix [Route](actix_web::Route) that listens for a `GET` request and tries
/// to route it using [leptos_router], asynchronously rendering an HTML page after all
/// `async` [Resource](leptos::Resource)s have loaded.
///
/// The provides a [MetaContext] and a [RouterIntegrationContext] to the app’s context before
/// rendering it, and includes any meta tags injected using [leptos_meta].
///
/// The HTML stream is rendered using [render_to_string_async](leptos::ssr::render_to_string_async),
/// and includes everything described in the documentation for that function.
///
/// This can then be set up at an appropriate route in your application:
/// ```
/// use std::{env, net::SocketAddr};
///
/// use actix_web::{App, HttpServer};
/// use leptos::*;
/// use leptos_router::Method;
///
/// #[component]
/// fn MyApp(cx: Scope) -> impl IntoView {
///     view! { cx, <main>"Hello, world!"</main> }
/// }
///
/// # if false { // don't actually try to run a server in a doctest...
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
///     let addr = conf.leptos_options.site_addr.clone();
///     HttpServer::new(move || {
///         let leptos_options = &conf.leptos_options;
///
///         App::new()
///             // {tail:.*} passes the remainder of the URL as the route
///             // the actual routing will be handled by `leptos_router`
///             .route(
///                 "/{tail:.*}",
///                 leptos_actix::render_app_async(
///                     leptos_options.to_owned(),
///                     |cx| view! { cx, <MyApp/> },
///                     Method::Get,
///                 ),
///             )
///     })
///     .bind(&addr)?
///     .run()
///     .await
/// }
/// # }
/// ```
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
/// - [MetaContext](leptos_meta::MetaContext)
/// - [RouterIntegrationContext](leptos_router::RouterIntegrationContext)
pub fn render_app_async<IV>(
    options: LeptosOptions,
    app_fn: impl Fn(leptos::Scope) -> IV + Clone + 'static,
    method: Method,
) -> Route
where
    IV: IntoView,
{
    render_app_async_with_context(options, |_cx| {}, app_fn, method)
}

/// Returns an Actix [Route](actix_web::Route) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application.
///
/// This function allows you to provide additional information to Leptos for your route.
/// It could be used to pass in Path Info, Connection Info, or anything your heart desires.
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
/// - [MetaContext](leptos_meta::MetaContext)
/// - [RouterIntegrationContext](leptos_router::RouterIntegrationContext)
pub fn render_app_to_stream_with_context<IV>(
    options: LeptosOptions,
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
    app_fn: impl Fn(leptos::Scope) -> IV + Clone + 'static,
    method: Method,
) -> Route
where
    IV: IntoView,
{
    render_app_to_stream_with_context_and_replace_blocks(
        options,
        additional_context,
        app_fn,
        method,
        false,
    )
}

/// Returns an Actix [Route](actix_web::Route) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application.
///
/// This function allows you to provide additional information to Leptos for your route.
/// It could be used to pass in Path Info, Connection Info, or anything your heart desires.
///
/// `replace_blocks` additionally lets you specify whether `<Suspense/>` fragments that read
/// from blocking resources should be retrojected into the HTML that's initially served, rather
/// than dynamically inserting them with JavaScript on the client. This means you will have
/// better support if JavaScript is not enabled, in exchange for a marginally slower response time.
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
/// - [MetaContext](leptos_meta::MetaContext)
/// - [RouterIntegrationContext](leptos_router::RouterIntegrationContext)
pub fn render_app_to_stream_with_context_and_replace_blocks<IV>(
    options: LeptosOptions,
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
    app_fn: impl Fn(leptos::Scope) -> IV + Clone + 'static,
    method: Method,
    replace_blocks: bool,
) -> Route
where
    IV: IntoView,
{
    let handler = move |req: HttpRequest| {
        let options = options.clone();
        let app_fn = app_fn.clone();
        let additional_context = additional_context.clone();
        let res_options = ResponseOptions::default();

        async move {
            let app = {
                let app_fn = app_fn.clone();
                let res_options = res_options.clone();
                move |cx| {
                    provide_contexts(cx, &req, res_options);
                    (app_fn)(cx).into_view(cx)
                }
            };

            stream_app(
                &options,
                app,
                res_options,
                additional_context,
                replace_blocks,
            )
            .await
        }
    };
    match method {
        Method::Get => web::get().to(handler),
        Method::Post => web::post().to(handler),
        Method::Put => web::put().to(handler),
        Method::Delete => web::delete().to(handler),
        Method::Patch => web::patch().to(handler),
    }
}

/// Returns an Actix [Route](actix_web::Route) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an in-order HTML stream of your application.
///
/// This function allows you to provide additional information to Leptos for your route.
/// It could be used to pass in Path Info, Connection Info, or anything your heart desires.
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
/// - [MetaContext](leptos_meta::MetaContext)
/// - [RouterIntegrationContext](leptos_router::RouterIntegrationContext)
pub fn render_app_to_stream_in_order_with_context<IV>(
    options: LeptosOptions,
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
    app_fn: impl Fn(leptos::Scope) -> IV + Clone + 'static,
    method: Method,
) -> Route
where
    IV: IntoView,
{
    let handler = move |req: HttpRequest| {
        let options = options.clone();
        let app_fn = app_fn.clone();
        let additional_context = additional_context.clone();
        let res_options = ResponseOptions::default();

        async move {
            let app = {
                let app_fn = app_fn.clone();
                let res_options = res_options.clone();
                move |cx| {
                    provide_contexts(cx, &req, res_options);
                    (app_fn)(cx).into_view(cx)
                }
            };

            stream_app_in_order(&options, app, res_options, additional_context).await
        }
    };
    match method {
        Method::Get => web::get().to(handler),
        Method::Post => web::post().to(handler),
        Method::Put => web::put().to(handler),
        Method::Delete => web::delete().to(handler),
        Method::Patch => web::patch().to(handler),
    }
}

/// Returns an Actix [Route](actix_web::Route) that listens for a `GET` request and tries
/// to route it using [leptos_router], asynchronously serving the page once all `async`
/// [Resource](leptos::Resource)s have loaded.
///
/// This function allows you to provide additional information to Leptos for your route.
/// It could be used to pass in Path Info, Connection Info, or anything your heart desires.
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [ResponseOptions]
/// - [HttpRequest](actix_web::HttpRequest)
/// - [MetaContext](leptos_meta::MetaContext)
/// - [RouterIntegrationContext](leptos_router::RouterIntegrationContext)
pub fn render_app_async_with_context<IV>(
    options: LeptosOptions,
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
    app_fn: impl Fn(leptos::Scope) -> IV + Clone + 'static,
    method: Method,
) -> Route
where
    IV: IntoView,
{
    let handler = move |req: HttpRequest| {
        let options = options.clone();
        let app_fn = app_fn.clone();
        let additional_context = additional_context.clone();
        let res_options = ResponseOptions::default();

        async move {
            let app = {
                let app_fn = app_fn.clone();
                let res_options = res_options.clone();
                move |cx| {
                    provide_contexts(cx, &req, res_options);
                    (app_fn)(cx).into_view(cx)
                }
            };

            render_app_async_helper(&options, app, res_options, additional_context).await
        }
    };
    match method {
        Method::Get => web::get().to(handler),
        Method::Post => web::post().to(handler),
        Method::Put => web::put().to(handler),
        Method::Delete => web::delete().to(handler),
        Method::Patch => web::patch().to(handler),
    }
}

fn provide_contexts(cx: leptos::Scope, req: &HttpRequest, res_options: ResponseOptions) {
    let path = leptos_corrected_path(req);

    let integration = ServerIntegration { path };
    provide_context(cx, RouterIntegrationContext::new(integration));
    provide_context(cx, MetaContext::new());
    provide_context(cx, res_options);
    provide_context(cx, req.clone());
    provide_server_redirect(cx, move |path| leptos_actix::redirect(cx, path));
}

fn leptos_corrected_path(req: &HttpRequest) -> String {
    let path = req.path();
    let query = req.query_string();
    if query.is_empty() {
        "http://leptos".to_string() + path
    } else {
        "http://leptos".to_string() + path + "?" + query
    }
}

async fn stream_app(
    options: &LeptosOptions,
    app: impl FnOnce(leptos::Scope) -> View + 'static,
    res_options: ResponseOptions,
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
    replace_blocks: bool,
) -> HttpResponse<BoxBody> {
    let (stream, runtime, scope) =
        render_to_stream_with_prefix_undisposed_with_context_and_block_replacement(
            app,
            move |cx| generate_head_metadata_separated(cx).1.into(),
            additional_context,
            replace_blocks,
        );

    build_stream_response(options, res_options, stream, runtime, scope).await
}

async fn stream_app_in_order(
    options: &LeptosOptions,
    app: impl FnOnce(leptos::Scope) -> View + 'static,
    res_options: ResponseOptions,
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
) -> HttpResponse<BoxBody> {
    let (stream, runtime, scope) =
        leptos::ssr::render_to_stream_in_order_with_prefix_undisposed_with_context(
            app,
            move |cx| generate_head_metadata_separated(cx).1.into(),
            additional_context,
        );

    build_stream_response(options, res_options, stream, runtime, scope).await
}

async fn build_stream_response(
    options: &LeptosOptions,
    res_options: ResponseOptions,
    stream: impl Stream<Item = String> + 'static,
    runtime: RuntimeId,
    scope: ScopeId,
) -> HttpResponse {
    let cx = leptos::Scope { runtime, id: scope };
    let mut stream = Box::pin(stream);

    // wait for any blocking resources to load before pulling metadata
    let first_app_chunk = stream.next().await.unwrap_or_default();

    let (head, tail) = html_parts_separated(options, use_context::<MetaContext>(cx).as_ref());

    let mut stream = Box::pin(
        futures::stream::once(async move { head.clone() })
            .chain(futures::stream::once(async move { first_app_chunk }).chain(stream))
            .chain(futures::stream::once(async move {
                runtime.dispose();
                tail.to_string()
            }))
            .map(|html| Ok(web::Bytes::from(html)) as Result<web::Bytes>),
    );

    // Get the first and second in the stream, which renders the app shell, and thus allows
    // Resources to run
    let first_chunk = stream.next().await;
    let second_chunk = stream.next().await;

    let res_options = res_options.0.read();

    let (status, headers) = (res_options.status, res_options.headers.clone());
    let status = status.unwrap_or_default();

    let complete_stream =
        futures::stream::iter([first_chunk.unwrap(), second_chunk.unwrap()]).chain(stream);
    let mut res = HttpResponse::Ok()
        .content_type("text/html")
        .streaming(complete_stream);

    // Add headers manipulated in the response
    for (key, value) in headers.into_iter() {
        res.headers_mut().append(key, value);
    }

    // Set status to what is returned in the function
    let res_status = res.status_mut();
    *res_status = status;
    // Return the response
    res
}

async fn render_app_async_helper(
    options: &LeptosOptions,
    app: impl FnOnce(leptos::Scope) -> View + 'static,
    res_options: ResponseOptions,
    additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
) -> HttpResponse<BoxBody> {
    let (stream, runtime, scope) =
        leptos::ssr::render_to_stream_in_order_with_prefix_undisposed_with_context(
            app,
            move |_| "".into(),
            additional_context,
        );

    let html = build_async_response(stream, options, runtime, scope).await;

    let res_options = res_options.0.read();

    let (status, headers) = (res_options.status, res_options.headers.clone());
    let status = status.unwrap_or_default();

    let mut res = HttpResponse::Ok().content_type("text/html").body(html);

    // Add headers manipulated in the response
    for (key, value) in headers.into_iter() {
        res.headers_mut().append(key, value);
    }

    // Set status to what is returned in the function
    let res_status = res.status_mut();
    *res_status = status;
    // Return the response
    res
}

pub trait WebPortalRoutes {
    fn leptos_routes<IV>(
        self,
        options: LeptosOptions,
        paths: Vec<RouteListing>,
        app_fn: impl Fn(leptos::Scope) -> IV + Clone + Send + 'static,
    ) -> Self
    where
        IV: IntoView + 'static;

    fn leptos_routes_with_context<IV>(
        self,
        options: LeptosOptions,
        paths: Vec<RouteListing>,
        additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
        app_fn: impl Fn(leptos::Scope) -> IV + Clone + Send + 'static,
    ) -> Self
    where
        IV: IntoView + 'static;
}

impl<T> WebPortalRoutes for actix_web::App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    fn leptos_routes<IV>(
        self,
        options: LeptosOptions,
        paths: Vec<RouteListing>,
        app_fn: impl Fn(leptos::Scope) -> IV + Clone + Send + 'static,
    ) -> Self
    where
        IV: IntoView + 'static,
    {
        self.leptos_routes_with_context(options, paths, |_| {}, app_fn)
    }

    fn leptos_routes_with_context<IV>(
        self,
        options: LeptosOptions,
        paths: Vec<RouteListing>,
        additional_context: impl Fn(leptos::Scope) + 'static + Clone + Send,
        app_fn: impl Fn(leptos::Scope) -> IV + Clone + Send + 'static,
    ) -> Self
    where
        IV: IntoView + 'static,
    {
        let mut router = self;
        for listing in paths.iter() {
            let path = listing.path();
            let mode = listing.mode();

            for method in listing.methods() {
                router = router.route(
                    path,
                    match mode {
                        SsrMode::OutOfOrder => render_app_to_stream_with_context(
                            options.clone(),
                            additional_context.clone(),
                            app_fn.clone(),
                            method,
                        ),
                        SsrMode::PartiallyBlocked => {
                            render_app_to_stream_with_context_and_replace_blocks(
                                options.clone(),
                                additional_context.clone(),
                                app_fn.clone(),
                                method,
                                true,
                            )
                        }
                        SsrMode::InOrder => render_app_to_stream_in_order_with_context(
                            options.clone(),
                            additional_context.clone(),
                            app_fn.clone(),
                            method,
                        ),
                        SsrMode::Async => render_app_async_with_context(
                            options.clone(),
                            additional_context.clone(),
                            app_fn.clone(),
                            method,
                        ),
                    },
                );
            }
        }
        router
    }
}
