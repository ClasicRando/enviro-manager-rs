use leptos::*;

#[component]
pub fn toast<S>(cx: Scope, body: S) -> impl IntoView
where
    S: Into<String>,
{
    view! { cx,
        <div class="toast fade show" role="alert" aria-live="assertive" aria-atomic="true">
            <div class="toast-header">
                <img src="/assets/bell_icon.png" width="20" class="me-1"/>
                <strong class="me-auto">"EnviroManager"</strong>
                <button type="button" class="btn-close" onclick="closeToast(this)" aria-label="Close"></button>
            </div>
            <div class="toast-body">{body.into()}</div>
        </div>
    }
}
