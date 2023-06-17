use leptos::*;
use leptos_router::*;
use web_portal_common::User;

use crate::Page;

#[component]
pub fn Home(cx: Scope, user_info: Signal<Option<User>>) -> impl IntoView {
    view! { cx,
        <h2>"EnviroManger"</h2>
        {move || match user_info.get() {
            Some(info) => {
                view! { cx, <p>"You are logged in as " {info.full_name} "."</p> }
                    .into_view(cx)
            }
            None => {
                view! { cx,
                    <p>"You are not logged in."</p>
                    <A href=Page::Login.path()>"Login now."</A>
                }
                    .into_view(cx)
            }
        }}
    }
}
