#[cfg(feature = "ssr")]
pub mod api;
#[cfg(feature = "ssr")]
pub mod app;
#[cfg(not(feature = "ssr"))]
pub mod app {
    use leptos::*;
    use leptos_meta::*;
    use leptos_router::*;

    #[component]
    pub fn App(cx: Scope) -> impl IntoView {
        // Provides context that manages stylesheets, titles, meta tags, etc.
        provide_meta_context(cx);

        view! {
            cx,
            <Meta charset="utf-8"/>
            <Meta name="viewport" content="width=device-width, initial-scale=1"/>
            <Meta name="theme-color" content="#000000"/>
            // injects a stylesheet into the document <head>
            // id=leptos means cargo-leptos will hot-reload this stylesheet
            <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

            // sets the document title
            <Title text="EnviroManager"/>
            <Link
                rel="stylesheet"
                href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"
                integrity="sha384-9ndCyUaIbzAi2FUVXJi0CjmCapSmO7SnpJef0486qhLnuZ2cdeRhO02iuK6FUUVM"
                crossorigin="anonymous"
            />

            // content for this welcome page
            <Router>
                <main>
                    <Routes>
                        <Route path="" view=|cx| view! { cx, <p>"Client side rendering not available"</p> }/>
                    </Routes>
                </main>
            </Router>
        }
    }
}
#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod components;
#[cfg(feature = "ssr")]
pub mod pages;
#[cfg(feature = "ssr")]
pub mod routes;
use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "hydrate")] {

  use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
      use app::*;
      use leptos::*;

      console_error_panic_hook::set_once();

      leptos::mount_to_body(move |cx| {
          view! { cx, <App/> }
      });
    }
}
}
