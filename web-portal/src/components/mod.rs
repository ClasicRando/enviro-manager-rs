pub mod base;
pub mod data_display;
pub mod error;
pub mod grid;
pub mod login;
pub mod modal;
pub mod table;
pub mod users;
pub mod workflow_engine;

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
