use leptos::*;

#[component]
pub fn LoginForm(cx: Scope) -> impl IntoView {
    view! { cx,
        <h3 class="login-form mx-auto">"Login to EnviroManager"</h3>
        <form id="loginForm" class="login-form mx-auto" hx-post="/api/login"
            hx-target="#errorMessage" hx-swap="innerHTML">
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
    }
}
