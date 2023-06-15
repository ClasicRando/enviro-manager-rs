use leptos::*;
use leptos_router::*;

#[component]
pub fn CredentialsForm(cx: Scope) -> impl IntoView {
    view! { cx,
        <main>
            <h3>"Login to EnviroManager"</h3>
            <Form action="">
                <div class="form-group">
                <label for="username">"Username"</label>
                <input
                    class="form-control"
                    type="text"
                    id="username"
                    name="username"
                    required={true}
                />
                </div>
                <div class="form-group">
                <label for="password">"Password"</label>
                <input
                    class="form-control"
                    type="password"
                    id="password"
                    name="password"
                    required={true}
                />
                </div>
                <input class="btn btn-primary" value="Login" type="submit"/>
            </Form>
        </main>
    }
}
