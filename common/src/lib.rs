use std::path::{PathBuf, Path};

pub mod db_build;
pub mod db_test;

fn package_dir() -> PathBuf {
    Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf()
}

fn workspace_dir() -> PathBuf {
    package_dir().join("..")
}
