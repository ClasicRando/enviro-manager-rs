mod base;
mod data_display;
mod error;
mod grid;
mod login;
mod modal;
mod table;
mod toast;
mod users;
mod workflow_engine;

use std::fmt::Display;

/// Convert a value into into
fn into_view<T>(val: T) -> String
where
    T: Display,
{
    val.to_string()
}

/// Convert an [Option] into a value that can be shown in a view. If [None], default to "-". For a
/// specified default value see [into_view_option_default]
fn into_view_option<T>(val: Option<T>) -> String
where
    T: Display,
{
    into_view_option_default(val, "-")
}

/// Convert an [Option] into a value that can be show in a view. If [None], default to the specified
/// default
#[inline(always)]
fn into_view_option_default<T>(val: Option<T>, default: &'static str) -> String
where
    T: Display,
{
    if let Some(val) = val {
        val.to_string();
    }
    default.to_owned()
}

pub use base::BasePage;
pub use error::UserMissingRole;
pub use login::LoginForm;
pub use toast::Toast;

pub use self::{
    users::{EditUser, UsersTable},
    workflow_engine::{
        main_page::{
            default_workflow_engine_tab_url, ActiveExecutors, ActiveExecutorsTab,
            ActiveWorkflowRuns, ActiveWorkflowRunsTab,
        },
        workflow_run_page::WorkflowRunDisplay,
    },
};
