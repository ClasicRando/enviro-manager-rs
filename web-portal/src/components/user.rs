use leptos::*;

use super::base::BasePage;

#[component]
pub fn update_password(cx: Scope) -> impl IntoView {
    view! { cx,
        <form hx-post="/api/user/update-password">
            <div class="input-group">
                <span class="input-group-text">"New Password"</span>
                <input type="text" aria-label="new password name" name="new_password" class="form-control" />
            </div>
            <div class="input-group">
                <span class="input-group-text">"Current Password"</span>
                <input type="password" aria-label="current password" name="current_password" class="form-control" />
            </div>
            <input type="submit" class="btn btn-primary mt-2" value="Update"/>
        </form>
    }
}

#[component]
pub fn update_username(cx: Scope) -> impl IntoView {
    view! { cx,
        <form hx-post="/api/user/update-username">
            <div class="input-group">
                <span class="input-group-text">"New Username"</span>
                <input type="text" aria-label="new username name" name="new_username" class="form-control" />
            </div>
            <div class="input-group">
                <span class="input-group-text">"Current Password"</span>
                <input type="password" aria-label="current password" name="current_password" class="form-control" />
            </div>
            <input type="submit" class="btn btn-primary mt-2" value="Update"/>
        </form>
    }
}

#[component]
pub fn update_full_name(cx: Scope) -> impl IntoView {
    view! { cx,
        <form hx-post="/api/user/update-full-name">
            <div class="input-group">
                <span class="input-group-text">"Full Name"</span>
                <input type="text" aria-label="First name" name="new_first_name" class="form-control" />
                <input type="text" aria-label="Last name" name="new_last_name" class="form-control" />
            </div>
            <div class="input-group">
                <span class="input-group-text">"Current Password"</span>
                <input type="password" aria-label="current password" name="current_password" class="form-control" />
            </div>
            <input type="submit" class="btn btn-primary mt-2" value="Update"/>
        </form>
    }
}

#[component]
pub fn user(cx: Scope, username: String) -> impl IntoView {
    view! { cx,
        <BasePage title="User" username=username>
            <div hx-swap="innerHTML" hx-target="#updateSection">
                <div class="form-check">
                    <input class="form-check-input" type="radio" name="updateUserOption" id="updateName" hx-get="/api/user/update-full-name" checked/>
                    <label class="form-check-label" for="updateName">
                    "Update Name"
                    </label>
                </div>
                <div class="form-check">
                    <input class="form-check-input" type="radio" name="updateUserOption" id="updateUsername" hx-get="/api/user/update-username"/>
                    <label class="form-check-label" for="updateUsername">
                    "Update Username"
                    </label>
                </div>
                <div class="form-check">
                    <input class="form-check-input" type="radio" name="updateUserOption" id="updatePassword" hx-get="/api/user/update-password"/>
                    <label class="form-check-label" for="updatePassword">
                    "Update Password"
                    </label>
                </div>
            </div>
            <div id="updateSection" class="mt-3 w-50">
                <UpdateFullName />
            </div>
        </BasePage>
    }
}
