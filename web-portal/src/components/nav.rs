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
fn user_context(cx: Scope, username: String) -> impl IntoView {
    if username.is_empty() {
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
                {username}
            </a>
            <ul class="dropdown-menu">
                <li><a class="dropdown-item" href="/logout">"Logout"</a></li>
            </ul>
        </li>
    }
}

#[component]
pub fn nav(cx: Scope, username: String) -> impl IntoView {
    view! { cx,
        <nav class="navbar navbar-expand-lg bg-body-tertiary">
            <div class="container-md">
                <a class="navbar-brand" href="#">"EnviroManager"</a>
                <button class="navbar-toggler collapsed" type="button" data-bs-toggle="collapse"
                    data-bs-target="#navbarScroll" aria-controls="navbarScroll" aria-expanded="false"
                    aria-label="Toggle navigation">
                    <span class="navbar-toggler-icon"></span>
                </button>
                <div class="collapse navbar-collapse" id="navbarScroll">
                    <ul class="navbar-nav me-auto my-2 my-lg-0 navbar-nav-scroll" style="--bs-scroll-height: 100px;">
                        <li class="nav-item">
                            <a class="nav-link" href="/">"Home"</a>
                        </li>
                        <li class="nav-item">
                            <a class="nav-link" href="/workflow-engine">"Workflow Engine"</a>
                        </li>
                    </ul>

                    <ul class="navbar-nav ms-auto my-2 my-lg-0 navbar-nav-scroll" style="--bs-scroll-height: 100px;">
                        <UserContext username=username/>
                        <ThemeSelector />
                    </ul>
                </div>
            </div>
        </nav>
    }
}
