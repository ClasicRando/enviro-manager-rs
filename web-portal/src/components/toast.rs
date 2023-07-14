use leptos::*;

#[component]
pub fn toast(cx: Scope, body: String) -> impl IntoView {
    view! { cx,
        <div class="toast fade show" role="alert" aria-live="assertive" aria-atomic="true">
            <div class="toast-header">
                <img src="/assets/bell_icon.png" width="20" class="me-1"/>
                <strong class="me-auto">"EnviroManager"</strong>
                <button type="button" class="btn-close" data-bs-dismiss="toast" aria-label="Close"></button>
            </div>
            <div class="toast-body">{body}</div>
        </div>
    }
}

#[component]
pub fn request_toast<S>(cx: Scope, body: S) -> impl IntoView
where
    S: Into<String>,
{
    view! { cx,
        <div class="d-none" hx-target="#toasts" hx-swap="beforeend">
            <input hx-trigger="DOMNodeInserted" hx-post="/api/toast" name="message" value=body.into()/>
        </div>
    }
}

#[macro_export]
macro_rules! build_toast {
    ($message:literal) => {{
        let html = leptos::ssr::render_to_string(|cx| {
            leptos::view! { cx,
                <div class="d-none" hx-target="#toasts" hx-swap="beforeend">
                    <input hx-trigger="DOMNodeInserted" hx-get="/api/toast" value=$message.to_owned()>
                </div>
            }
        });
        $crate::utils::html_chunk!(html)
    }};

    ($message:ident) => {{
        let html = leptos::ssr::render_to_string(|cx| {
            leptos::view! { cx,
                <div class="d-none" hx-target="#toasts" hx-swap="beforeend">
                    <input hx-trigger="DOMNodeInserted" hx-get="/api/toast" value=$message>
                </div>
            }
        });
        $crate::utils::html_chunk!(html)
    }};
}

#[macro_export]
macro_rules! toast {
    ($message:literal) => {{
        use $crate::components::Toast;
        let html = leptos::ssr::render_to_string(|cx| {
            leptos::view! { cx, <Toast body=$message.to_owned()/> }
        });
        $crate::utils::html_chunk!(html)
    }};

    ($message:ident) => {{
        use $crate::components::Toast;
        let html = leptos::ssr::render_to_string(|cx| {
            leptos::view! { cx, <Toast body=$message/> }
        });
        $crate::utils::html_chunk!(html)
    }};
}

#[macro_export]
macro_rules! error_toast {
    ($error:ident, $message:literal) => {{
        use $crate::components::Toast;
        log::error!("{}", $error);
        let html = leptos::ssr::render_to_string(|cx| {
            leptos::view! { cx, <Toast body=$message.to_owned()/> }
        });
        $crate::utils::html_chunk!(html)
    }};

    ($error:ident, $message:ident) => {{
        use $crate::components::Toast;
        log::error!("{}", $error);
        let html = leptos::ssr::render_to_string(|cx| {
            leptos::view! { cx, <Toast body=$message/> }
        });
        $crate::utils::html_chunk!(html)
    }};
}

pub use build_toast;
pub use error_toast;
pub use toast;
