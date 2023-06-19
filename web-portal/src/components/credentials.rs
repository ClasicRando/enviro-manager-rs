use leptos::*;
use leptos_router::*;

#[component]
pub fn CredentialsForm(
    cx: Scope,
    action: Action<(String, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (email, set_email) = create_signal(cx, String::new());
    let (password, set_password) = create_signal(cx, String::new());
    let dispatch_action = move || action.dispatch((email.get(), password.get()));

    let submit_is_disabled = Signal::derive(cx, move || {
        disabled.get() || email.get().is_empty() || password.get().is_empty()
    });

    view! { cx,
        <main>
            <h3>"Login to EnviroManager"</h3>
            <Form on:submit=|ev| ev.prevent_default() action="">
                <div class="form-group">
                <label for="username">"Username"</label>
                <input
                    class="form-control"
                    type="text"
                    id="username"
                    name="username"
                    required
                    on:keyup=move |ev: ev::KeyboardEvent| {
                        let val = event_target_value(&ev);
                        set_email.update(|v| *v = val);
                    }
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_email.update(|v| *v = val);
                    }
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
                    on:keyup=move |ev: ev::KeyboardEvent| {
                        let val = event_target_value(&ev);
                        set_password.update(|v| *v = val);
                    }
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_password.update(|v| *v = val);
                    }
                />
                </div>
                {move || {
                    error
                        .get()
                        .map(|err| {
                            view! { cx, <p style="color:red;">{err}</p> }
                        })
                }}
                <input
                    class="btn btn-primary"
                    value="Login"
                    type="submit"
                    prop:disabled=move || submit_is_disabled.get()
                    on:click=move |_| dispatch_action()
                />
            </Form>
        </main>
    }
}
