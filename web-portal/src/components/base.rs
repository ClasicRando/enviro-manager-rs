use leptos::*;
use users::data::{role::RoleName, user::User};

#[component]
fn ThemeSelector(cx: Scope) -> impl IntoView {
    view! { cx,
        <li class="nav-item dropdown">
            <button class="btn btn-link nav-link py-2 px-0 px-lg-2 dropdown-toggle align-items-center"
                id="bd-theme" type="button" aria-expanded="false" data-bs-toggle="dropdown"
                data-bs-display="static" aria-label="Toggle theme (dark)"
            >
                <i class="fa-solid fa-moon me-1" id="theme-selector"></i>
                <span class="d-lg-none ms-2" id="bd-theme-text">"Toggle theme"</span>
            </button>
            <ul class="dropdown-menu dropdown-menu-end" aria-labelledby="bd-theme-text">
                <li>
                    <button type="button" class="dropdown-item d-flex align-items-center"
                        data-bs-theme-value="light" aria-pressed="false">
                        <i class="fa-solid fa-sun me-1"></i>
                        "Light"
                    </button>
                </li>
                <li>
                    <button type="button" class="dropdown-item d-flex align-items-center active"
                        data-bs-theme-value="dark" aria-pressed="true">
                        <i class="fa-solid fa-moon me-1"></i>
                        "Dark"
                    </button>
                </li>
                <li>
                    <button type="button" class="dropdown-item d-flex align-items-center"
                        data-bs-theme-value="auto" aria-pressed="false">
                        <i class="fa-solid fa-circle-half-stroke me-1"></i>
                        "Auto"
                    </button>
                </li>
            </ul>
        </li>
    }
}

#[component]
fn UserContext(cx: Scope, user_full_name: String) -> impl IntoView {
    if user_full_name.is_empty() {
        return view! { cx,
            <li class="nav-item">
                <a class="nav-link disabled" href="#">"Login"</a>
            </li>
        };
    }
    view! { cx,
        <li class="nav-item dropdown">
            <a class="nav-link dropdown-toggle" href="#" role="button" aria-expanded="false">
                {user_full_name}
            </a>
            <ul class="dropdown-menu">
                <li><a class="dropdown-item" href="/logout" hx-boost="true">"Logout"</a></li>
            </ul>
        </li>
    }
}

#[component]
fn Nav(cx: Scope, #[prop(optional)] user: Option<User>) -> impl IntoView {
    let users_page = match user.as_ref().map(|u| u.check_role(RoleName::Admin)) {
        Some(Ok(_)) => Some(view! { cx,
            <li class="nav-item">
                <a class="nav-link" href="/users" hx-boost="true">"Users"</a>
            </li>
        }),
        Some(Err(_)) | None => None,
    };
    let user_context = user.map(|u| view! { cx, <UserContext user_full_name=u.full_name/> });
    view! { cx,
        <nav class="navbar navbar-expand-lg bg-body-tertiary" id="mainNavBar">
            <div class="container-fluid">
                <a class="navbar-brand" href="#">
                    <img src="/assets/favicon.ico" alt="logo" class="d-inline-block align-text-top me-1" height="30"/>
                    "EnviroManager"
                </a>
                <ul class="navbar-nav me-auto my-2 my-lg-0 navbar-nav-scroll" style="--bs-scroll-height: 100px;">
                    <li class="nav-item">
                        <a class="nav-link" href="/" hx-boost="true">"Home"</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/workflow-engine" hx-boost="true">"Workflow Engine"</a>
                    </li>
                    {users_page}
                </ul>
                <ul class="navbar-nav ms-auto my-2 my-lg-0 navbar-nav-scroll" style="--bs-scroll-height: 100px;">
                    {user_context}
                    <ThemeSelector />
                </ul>
            </div>
        </nav>
    }
}

#[component]
pub fn BasePage(
    cx: Scope,
    title: &'static str,
    #[prop(optional)] user: Option<User>,
    #[prop(optional)] stylesheet_href: &'static str,
    #[prop(optional)] script_src: &'static str,
    #[prop(optional)] children: Option<Children>,
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
                <script defer src="/assets/fontawesome/js/solid.js"></script>
                <script defer src="/assets/fontawesome/js/fontawesome.js"></script>
                <title>"EnviroManager - "{title}</title>
            </head>
            <noscript>"Javascript must be enabled for most site features to work"</noscript>
            <body class="p-3 m-0 border-0">
                <div class="container-fluid">
                    {nav}
                    {children.map(|f| f(cx))}
                </div>
                <div class="toast-container top-0 end-0 p-3" id="toasts"></div>
                <div id="modals"></div>
                {page_script}
            </body>
        </html>
    }
}
//<script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js" integrity="sha384-geWF76RCwLtnZ8qwWowPQNguL3RmwHVBC9FhGdlKrxdiJJigb/j/68SIy3Te4Bkz" crossorigin="anonymous"></script>
