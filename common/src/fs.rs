//! Module containing file system utilities

use std::path::{Path, PathBuf};

use tokio::{fs::File, io::AsyncReadExt};

use crate::error::EmResult;

/// Returns a [PathBuf] pointing to the directory of the current package. Utilizes the
/// 'CARGO_MANIFEST_DIR' cargo environment variable.
/// # Errors
/// This function will return an error if the `CARGO_MANIFEST_DIR` environment variable is not set
#[cfg(feature = "utils")]
pub fn package_dir() -> EmResult<PathBuf> {
    Ok(Path::new(&std::env::var("CARGO_MANIFEST_DIR")?).to_path_buf())
}

/// Returns a [PathBuf] pointing to the current workspace. Fetches the package directory using
/// [package_dir] then navigates to the parent directory to find the workspace.
/// # Errors
/// This function will return an error if the `CARGO_MANIFEST_DIR` environment variable is not set
#[cfg(feature = "utils")]
pub fn workspace_dir() -> EmResult<PathBuf> {
    Ok(package_dir()?.join(".."))
}

/// Read the specified file using the `path` provided, returning the contents as a single [String]
/// buffer.
/// # Errors
/// This function will return an error if the file could not be opened or the contents of the file
/// could not be read into a [String] buffer.
#[cfg(feature = "utils")]
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
