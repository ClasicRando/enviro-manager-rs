#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![warn(clippy::cloned_instead_of_copied)]
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::create_dir)]
#![warn(clippy::empty_structs_with_brackets)]
#![warn(clippy::equatable_if_let)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::expect_used)]
#![warn(clippy::fn_params_excessive_bools)]
#![warn(clippy::from_iter_instead_of_collect)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::inefficient_to_string)]
#![warn(clippy::manual_let_else)]
#![warn(clippy::manual_string_new)]
#![warn(clippy::match_on_vec_items)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::missing_assert_message)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::needless_collect)]
#![warn(clippy::needless_continue)]
#![warn(clippy::needless_for_each)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_if_let_else)]
#![warn(clippy::panic)]
#![warn(clippy::partial_pub_fields)]
#![warn(clippy::print_stdout)]
#![warn(clippy::pub_use)]
#![warn(clippy::string_to_string)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_slice)]
#![warn(clippy::too_many_arguments)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::unnecessary_box_returns)]
#![warn(clippy::unused_async)]
#![warn(clippy::unused_self)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::use_self)]
#![warn(clippy::wildcard_imports)]

//! Common components of the EnivroManager application suite

use std::path::{Path, PathBuf};

use tokio::{fs::File, io::AsyncReadExt};

use crate::error::EmResult;

pub mod api;
pub mod database;
pub mod email;
pub mod error;

/// Returns a [PathBuf] pointing to the directory of the current package. Utilizes the
/// 'CARGO_MANIFEST_DIR' cargo environment variable.
/// # Errors
/// This function will return an error if the `CARGO_MANIFEST_DIR` environment variable is not set
pub fn package_dir() -> EmResult<PathBuf> {
    Ok(Path::new(&std::env::var("CARGO_MANIFEST_DIR")?).to_path_buf())
}

/// Returns a [PathBuf] pointing to the current workspace. Fetches the package directory using
/// [package_dir] then navigates to the parent directory to find the workspace.
/// # Errors
/// This function will return an error if the `CARGO_MANIFEST_DIR` environment variable is not set
fn workspace_dir() -> EmResult<PathBuf> {
    Ok(package_dir()?.join(".."))
}

/// Read the specified file using the `path` provided, returning the contents as a single [String]
/// buffer.
/// # Errors
/// This function will return an error if the file could not be opened or the contents of the file
/// could not be read into a [String] buffer.
pub async fn read_file<P: AsRef<Path> + Send>(path: P) -> EmResult<String> {
    let path = path.as_ref();
    let mut file = match File::open(path).await {
        Ok(inner) => inner,
        Err(error) => return Err(format!("Could not open file, {:?}. {}", path, error).into()),
    };
    let mut block = String::new();
    file.read_to_string(&mut block).await?;
    Ok(block)
}
