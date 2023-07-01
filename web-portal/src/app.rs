use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::{
    api::user::{get_user, LoginUser},
    pages::{home::*, login::Login},
};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let login = create_server_action::<LoginUser>(cx);

    let user = create_resource(cx, move || (login.version().get(),), move |_| get_user(cx));

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,
        <Meta charset="utf-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Meta name="theme-color" content="#000000"/>
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/web_portal.css"/>

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
            <header>
                <Transition
                    fallback=move || view! {cx, <span>"Loading..."</span>}
                >
                {move || {
                    user.read(cx).map(|user| match user {
                        Err(e) => {
                            log::error!("{}", e);
                            view! { cx, }.into_view(cx)
                        }
                        Ok(None) => view! { cx, <span>"Not logged in"</span> }.into_view(cx),
                        Ok(Some(user)) => view! {cx,
                            <span>{format!("Logged in as: {}", user.full_name())}</span>
                        }.into_view(cx)
                    })
                }}
                </Transition>
            </header>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                    <Route path="login" view=move |cx| view! { cx, <Login action=login/> }/>
                </Routes>
            </main>
        </Router>
    }
}
