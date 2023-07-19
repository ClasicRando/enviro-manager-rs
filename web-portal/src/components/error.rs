use leptos::*;
use users::data::{role::RoleName, user::User};

use crate::components::base::BasePage;

#[component]
pub fn UserMissingRole(cx: Scope, user: User, missing_role: RoleName) -> impl IntoView {
    let role_name: &'static str = missing_role.into();
    view! { cx,
        <BasePage title="Missing Role" user=user>
            <div>{format!("User cannot access this page because they are missing the role: {role_name}")}</div>
        </BasePage>
    }
}
