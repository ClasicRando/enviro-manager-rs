use leptos::*;
use leptos_router::*;

use crate::api::user::LoginUser;

#[component]
pub fn Login(cx: Scope) -> impl IntoView {
    let (login_error, set_login_error) = create_signal(cx, None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(cx, false);
    let login = create_server_action::<LoginUser>(cx);
    let value = login.value();
    let disabled = Signal::derive(cx, move || wait_for_response.get());
    view! { cx,
        <h3>"Login to EnviroManager"</h3>
        <ActionForm action=login>
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
            {move || {
                login_error
                    .get()
                    .map(|err| {
                        view! { cx, <p style="color:red;">{err}</p> }
                    })
            }}
            <input
                class="btn btn-primary"
                value="Login"
                type="submit"
                prop:disabled=move || disabled.get()
            />
        </ActionForm>
    }
}
