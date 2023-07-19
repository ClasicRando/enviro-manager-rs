use leptos::*;
use users::data::user::User;

use super::base::BasePage;

#[component]
pub fn Index(cx: Scope, user: User) -> impl IntoView {
    view! { cx,
        <BasePage title="Index" user=user>
            <p>"Index"</p>
        </BasePage>
    }
}
