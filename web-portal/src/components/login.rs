use leptos::*;

use super::BasePage;

#[component]
pub fn login(cx: Scope) -> impl IntoView {
    view! { cx,
        <BasePage
            title="Index"
            stylesheet_href="/assets/login.css"
        >
            <h3 class="mx-auto">"Login to EnviroManager"</h3>
            <form id="loginForm" class="login-form mx-auto" hx-post="/api/htmx/login"
                hx-encoding="multipart/form-data" hx-target="#errorMessage" hx-swap="innerHTML">
                <div class="form-group">
                    <label for="username">"Username"</label>
                    <input class="form-control" type="text" id="username" name="username" required />
                </div>
                <div class="form-group">
                    <label for="password">"Password"</label>
                    <input class="form-control" type="password" id="password" name="password" required />
                </div>
                <div id="errorMessage"></div>
                <input class="btn btn-primary" value="Login" type="submit" />
            </form>
        </BasePage>
    }
}
