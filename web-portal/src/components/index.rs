use leptos::*;

use super::base::BasePage;

#[component]
pub fn index(cx: Scope, username: String) -> impl IntoView {
    view! { cx,
        <BasePage title="Index" username=username>
            <p>"Index"</p>
        </BasePage>
    }
}
