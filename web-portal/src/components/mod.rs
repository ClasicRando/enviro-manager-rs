mod base;
mod data_display;
mod error;
mod grid;
mod index;
mod login;
mod modal;
mod table;
mod toast;
mod users;
mod workflow_engine;

use std::fmt::Display;

use leptos::IntoView;

fn into_view<T: ToString>(val: T) -> impl IntoView {
    val.to_string()
}

fn display_option<T>(val: Option<T>) -> String
where
    T: Display,
{
    option_into_view_default(val, "-")
}

#[inline(always)]
fn option_into_view_default<T>(val: Option<T>, default: &'static str) -> String
where
    T: Display,
{
    if let Some(val) = val {
        return format!("{}", val);
    }
    default.to_owned()
}

pub use base::BasePage;
pub use error::UserMissingRole;
pub use index::Index;
pub use login::Login;
pub use toast::{build_toast, error_toast, toast, RequestToast, Toast};

pub use self::{
    users::{EditUser, UsersPage, UsersTable},
    workflow_engine::{
        main_page::{
            ActiveExecutors, ActiveExecutorsTab, ActiveWorkflowRuns, ActiveWorkflowRunsTab,
            WorkflowEngine,
        },
        workflow_run_page::{WorkflowRunDisplay, WorkflowRunTaskTable},
    },
};
