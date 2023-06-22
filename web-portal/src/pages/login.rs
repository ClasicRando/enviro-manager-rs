use leptos::*;
use leptos_router::*;

use crate::api::user::LoginUser;

#[component]
pub fn Login(cx: Scope, action: Action<LoginUser, Result<(), ServerFnError>>) -> impl IntoView {
    let disabled = action.pending();
    view! { cx,
        <ActionForm
            action=action
            class="login-form mx-auto"
        >
            <h3>"Login to EnviroManager"</h3>
            <div class="form-group">
            <label for="username">"Username"</label>
            <input
                class="form-control"
                type="text"
                id="username"
                name="username"
                required
            />
            </div>
            <div class="form-group">
            <label for="password">"Password"</label>
            <input
                class="form-control"
                type="password"
                id="password"
                name="password"
                required
            />
            </div>
            // {move || {
            //     login_error
            //         .get()
            //         .map(|err| {
            //             view! { cx, <p style="color:red;">{err}</p> }
            //         })
            // }}
            <input
                class="btn btn-primary"
                value="Login"
                type="submit"
                prop:disabled=move || disabled.get()
            />
        </ActionForm>
    }
}
