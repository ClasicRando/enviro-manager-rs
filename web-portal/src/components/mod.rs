mod base;
mod grid;
mod index;
mod login;
mod nav;
mod table;
mod user;
mod workflow_engine;

use leptos::IntoView;

fn into_view<T: ToString>(val: T) -> impl IntoView {
    val.to_string()
}

fn option_into_view<T: ToString>(val: Option<T>) -> impl IntoView {
    if let Some(val) = val {
        return val.to_string();
    }
    "-".to_string()
}

#[allow(unused)]
fn option_into_view_default<T: ToString>(val: Option<T>, default: &'static str) -> impl IntoView {
    if let Some(val) = val {
        return val.to_string();
    }
    default.to_string()
}

pub use index::Index;
pub use login::Login;
pub use user::{UpdateFullName, UpdatePassword, UpdateUsername, User};

pub use self::workflow_engine::{ActiveExecutors, ActiveWorkflowRuns, WorkflowEngine};
