use leptos::*;
use users::data::{
    role::{Role, RoleName},
    user::User,
};
use uuid::Uuid;

use crate::components::{
    base::BasePage,
    into_view,
    modal::EditModal,
    table::{DataTableExtras, ExtraTableButton, RowAction, RowWithDetails},
};

#[component]
fn UserRole(cx: Scope, role: Role) -> impl IntoView {
    let role_name: &'static str = role.name.into();
    view! { cx,
        <tr>
            <td>{role_name}</td>
            <td>{role.description}</td>
        </tr>
    }
}

#[component]
fn UserRow(cx: Scope, uid: Uuid, user: User) -> impl IntoView {
    let edit_url = move || format!("/api/users/edit/{}", user.uid);
    let actions = match user.check_role(RoleName::Admin) {
        Ok(_) if uid == user.uid => Some(
            view! { cx,
                <RowAction
                    title="Edit User"
                    api_url=edit_url()
                    icon="fa-edit"
                    target="#modals"
                    swap="beforeend"/>
            }
            .into_view(cx),
        ),
        Ok(_) => None,
        Err(_) => Some(
            view! { cx,
                <RowAction
                    title="Edit User"
                    api_url=edit_url()
                    icon="fa-edit"
                    target="#modals"
                    swap="beforeend"/>
                <RowAction
                    title="Add Role"
                    api_url=format!("/api/users/add-role/{}", user.uid)
                    icon="fa-plus"
                    target="#modals"
                    swap="beforeend"/>
            }
            .into_view(cx),
        ),
    };
    view! { cx,
        <RowWithDetails
            details_id=format!("{}", user.uid)
            details_header=view! { cx,
                <tr>
                    <th>"Name"</th>
                    <th>"Description"</th>
                </tr>
            }
            details=user.roles
            details_row_builder=|cx, role| view! { cx, <UserRole role=role/> }
            column_count=5
        >
            <td>{into_view(user.uid)}</td>
            <td>{user.username}</td>
            <td>{user.full_name}</td>
            <td>{actions}</td>
        </RowWithDetails>
    }
}

#[component]
pub fn UsersTable(cx: Scope, uid: Uuid, users: Vec<User>) -> impl IntoView {
    view! { cx,
        <DataTableExtras
            id="usersTable"
            caption="Users"
            header=view! { cx,
                <tr>
                    <th>"Roles"</th>
                    <th>"UUID"</th>
                    <th>"Username"</th>
                    <th>"Full Name"</th>
                    <th>"Actions"</th>
                </tr>
            }
            items=users
            row_builder=|cx, user| view! { cx, <UserRow uid=uid user=user/> }
            data_source="/api/users".to_owned()
            refresh=true
            search=true
            extra_buttons=vec![
                ExtraTableButton::new(
                    "Create new User",
                    "/api/users",
                    "fa-plus"
                ),
            ]/>
    }
}

#[component]
pub fn UsersPage(cx: Scope, user: User, users: Vec<User>) -> impl IntoView {
    let uid = user.uid;
    view! { cx,
        <BasePage title="Users" user=user>
            <UsersTable uid=uid users=users/>
        </BasePage>
    }
}

#[component]
pub fn EditUser(cx: Scope, user: User) -> impl IntoView {
    view! { cx,
        <EditModal
            id="editUser"
            title="Edit User"
            form=view! { cx,
                <div class="row mb-3">
                    <label for="username" class="col-sm-3 col-form-label">"Username"</label>
                    <div class="col-sm-9">
                        <input type="text" class="form-control" id="username" name="username"
                            value=user.username/>
                    </div>
                </div>
                <div class="row mb-3">
                    <label for="full_name" class="col-sm-3 col-form-label">"Full Name"</label>
                    <div class="col-sm-9">
                        <input type="text" class="form-control" id="full_name" name="full_name"
                            value=user.full_name/>
                    </div>
                </div>
            }
            patch_url=format!("/api/users/edit/{}", user.uid)
            target="#usersTableContainer".to_owned()/>
    }
}
