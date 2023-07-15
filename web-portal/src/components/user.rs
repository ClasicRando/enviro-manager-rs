use leptos::*;
use users::data::user::User;

use super::{
    base::BasePage,
    grid::{Col, Row},
};

#[component]
fn user_info_input<S>(
    cx: Scope,
    label: &'static str,
    name: &'static str,
    input_type: &'static str,
    value: S,
    edit: bool,
    #[prop(optional)] enabled: bool,
    is_editing: bool,
) -> impl IntoView
where
    S: Into<String>,
{
    let url_name = name.replace('_', "-");
    let button = if edit {
        view! { cx,
            <button class="btn btn-outline-secondary" type="submit" hx-post=format!("/api/user/edit-{url_name}")
                hx-include=".post-include">
                "Submit"
            </button>
            <button class="btn btn-outline-secondary" type="button" hx-get="/api/user/view-user">
                "Cancel"
            </button>
        }.into_view(cx)
    } else if is_editing {
        view! { cx, }.into_view(cx)
    } else {
        view! { cx,
            <button class="btn btn-outline-secondary" type="button" hx-get=format!("/api/user/edit-{url_name}")>
                <i class="fa-solid fa-edit"></i>
            </button>
        }.into_view(cx)
    };
    if edit || enabled {
        view! { cx,
            <div class="input-group">
                <span class="input-group-text">{label}</span>
                <input type=input_type class="post-include" aria-label=name name=name class="form-control" value=value.into()/>
                {button}
            </div>
        }
        .into_view(cx)
    } else {
        view! { cx,
            <div class="input-group">
                <span class="input-group-text">{label}</span>
                <input type=input_type aria-label=name name=name class="form-control" value=value.into() disabled/>
                {button}
            </div>
        }
        .into_view(cx)
    }
}

#[component]
pub fn user_info(cx: Scope, user: User, edit_section: UserEditSection) -> impl IntoView {
    let is_editing = edit_section != UserEditSection::None;
    let edit_name = edit_section == UserEditSection::FullName;
    let edit_username = edit_section == UserEditSection::Username;
    let current_password_input = if is_editing {
        view! { cx,
            <UserInfoInput
                label="Current Password"
                name="current_password"
                input_type="password"
                value=""
                edit=false
                enabled=true
                is_editing=true/>
        }
        .into_view(cx)
    } else {
        view! { cx, }.into_view(cx)
    };
    view! { cx,
        <form id="userInfo" hx-target="#userInfo" hx-swap="outerHTML">
            <UserInfoInput
                label="Name"
                name="full_name"
                input_type="text"
                value=user.full_name()
                edit=edit_name
                is_editing=is_editing/>
            <UserInfoInput
                label="Username"
                name="username"
                input_type="text"
                value=user.username()
                edit=edit_username
                is_editing=is_editing/>
            {current_password_input}
        </form>
    }
}

#[derive(PartialEq)]
pub enum UserEditSection {
    Username,
    FullName,
    None,
}

#[component]
pub fn user<S>(cx: Scope, user: User, #[prop(optional)] notification: S) -> impl IntoView
where
    S: Into<String> + Default,
{
    let notification = notification.into();
    let toast = if notification.is_empty() {
        view! { cx, }.into_view(cx)
    } else {
        view! { cx, <RequestToast body=message/> }.into_view(cx)
    };
    view! { cx,
        <BasePage title="User" user_full_name=user.full_name().to_owned()>
            <Row>
                <Col/>
                <Col>
                    <UserInfo user=user edit_section=UserEditSection::None/>
                </Col>
                <Col/>
            </Row>
            {toast}
        </BasePage>
    }
}
