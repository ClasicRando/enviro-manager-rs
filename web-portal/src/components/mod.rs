mod base;
mod data_display;
mod grid;
mod index;
mod login;
mod nav;
mod table;
mod toast;
mod workflow_engine;

use leptos::IntoView;

fn into_view<T: ToString>(val: T) -> impl IntoView {
    val.to_string()
}

fn display_option<T: ToString>(val: Option<T>) -> String {
    if let Some(val) = val {
        return val.to_string();
    }
    "-".to_owned()
}

#[allow(unused)]
fn option_into_view_default<T: ToString>(val: Option<T>, default: &'static str) -> impl IntoView {
    if let Some(val) = val {
        return val.to_string();
    }
    default.to_owned()
}

pub use index::Index;
pub use login::Login;
pub use toast::{build_toast, error_toast, toast, RequestToast, Toast};

pub use self::workflow_engine::{
    main_page::{ActiveExecutors, ActiveWorkflowRuns, WorkflowEngine},
    workflow_run_page::{WorkflowRunPage, WorkflowRunTaskTable},
};
