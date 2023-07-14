use leptos::*;

use super::base::BasePage;

#[component]
pub fn index(cx: Scope, user_full_name: String) -> impl IntoView {
    view! { cx,
        <BasePage title="Index" user_full_name=user_full_name>
            <p>"Index"</p>
        </BasePage>
    }
}
