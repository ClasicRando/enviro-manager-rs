use leptos::*;
use users::data::user::User;

use super::nav::Nav;

#[component]
pub fn BasePage(
    cx: Scope,
    title: &'static str,
    #[prop(optional)] user: Option<User>,
    #[prop(optional)] stylesheet_href: &'static str,
    #[prop(optional)] script_src: &'static str,
    children: Children,
) -> impl IntoView {
    let page_stylesheet = if !stylesheet_href.is_empty() {
        Some(view! { cx, <link rel="stylesheet" href=stylesheet_href /> })
    } else {
        None
    };
    let page_script = if !script_src.is_empty() {
        Some(view! { cx, <script type="module" src=script_src></script> })
    } else {
        None
    };
    let nav = match user {
        Some(user) => view! { cx, <Nav user=user/> },
        None => view! { cx, <Nav/> },
    };
    view! { cx,
        <html lang="en" data-bs-theme="dark">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="theme-color" content="#000000" />
                <link rel="icon" type="image/ico" href="/assets/favicon.ico" />
                <link rel="stylesheet" href="/assets/style.css" />
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"
                    integrity="sha384-9ndCyUaIbzAi2FUVXJi0CjmCapSmO7SnpJef0486qhLnuZ2cdeRhO02iuK6FUUVM" crossorigin="anonymous" />
                {page_stylesheet}
                <script src="/assets/htmx.min.js"></script>
                <script type="module" src="/assets/utils.js"></script>
                <script defer src="/assets/fontawesome/js/brands.js"></script>
                <script defer src="/assets/fontawesome/js/solid.js"></script>
                <script defer src="/assets/fontawesome/js/fontawesome.js"></script>
                <title>"EnviroManager - "{title}</title>
            </head>
            <body class="p-3 m-0 border-0">
                <div class="container-fluid">
                    {nav}
                    {children(cx)}
                </div>
                <div class="toast-container top-0 end-0 p-3" id="toasts">
                </div>
                <script src="https://cdn.jsdelivr.net/npm/@popperjs/core@2.11.8/dist/umd/popper.min.js"
                    integrity="sha384-I7E8VVD/ismYTF4hNIPjVp/Zjvgyol6VFvRkX/vR+Vc4jQkC+hVqc2pM8ODewa9r"
                    crossorigin="anonymous"></script>
                <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.min.js"
                    integrity="sha384-fbbOQedDUMZZ5KreZpsbe1LCZPVmfTnH7ois6mU1QK+m14rQ1l2bGBq41eYeM/fS"
                    crossorigin="anonymous"></script>
                {page_script}
            </body>
        </html>
    }
}
