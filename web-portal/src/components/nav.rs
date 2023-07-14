use leptos::*;

#[component]
fn theme_selector(cx: Scope) -> impl IntoView {
    view! { cx,
        <li class="nav-item dropdown">
            <button class="btn btn-link nav-link py-2 px-0 px-lg-2 dropdown-toggle align-items-center"
                id="bd-theme" type="button" aria-expanded="false" data-bs-toggle="dropdown"
                data-bs-display="static" aria-label="Toggle theme (dark)">
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
fn user_context(cx: Scope, user_full_name: String) -> impl IntoView {
    if user_full_name.is_empty() {
        return view! { cx,
            <li class="nav-item">
                <a class="nav-link disabled" href="#">"Login"</a>
            </li>
        };
    }
    view! { cx,
        <li class="nav-item dropdown">
            <a class="nav-link dropdown-toggle" href="#" role="button" data-bs-toggle="dropdown"
                aria-expanded="false">
                {user_full_name}
            </a>
            <ul class="dropdown-menu">
                <li><a class="dropdown-item" href="/user">"Info"</a></li>
                <li><a class="dropdown-item" href="/logout">"Logout"</a></li>
            </ul>
        </li>
    }
}

#[component]
pub fn nav(cx: Scope, user_full_name: String) -> impl IntoView {
    view! { cx,
        <nav class="navbar navbar-expand-lg bg-body-tertiary" id="mainNavBar">
            <div class="container-fluid">
                <a class="navbar-brand" href="#">
                    <img src="/assets/favicon.ico" alt="logo" class="d-inline-block align-text-top me-1" height="30"/>
                    "EnviroManager"
                </a>
                <ul class="navbar-nav me-auto my-2 my-lg-0 navbar-nav-scroll" style="--bs-scroll-height: 100px;">
                    <li class="nav-item">
                        <a class="nav-link" href="/">"Home"</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="/workflow-engine">"Workflow Engine"</a>
                    </li>
                </ul>
                <ul class="navbar-nav ms-auto my-2 my-lg-0 navbar-nav-scroll" style="--bs-scroll-height: 100px;">
                    <UserContext user_full_name=user_full_name/>
                    <ThemeSelector />
                </ul>
            </div>
        </nav>
    }
}
