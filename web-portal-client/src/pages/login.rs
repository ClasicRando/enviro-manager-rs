use leptos::*;

use crate::components::credentials::CredentialsForm;

#[component]
pub fn Login(cx: Scope) -> impl IntoView {
    view! { cx,
        <CredentialsForm />
    }
}
