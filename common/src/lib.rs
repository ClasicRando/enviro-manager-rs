use std::path::{PathBuf, Path};

use tokio::{fs::File, io::AsyncReadExt};

pub mod db_build;
pub mod db_test;

fn package_dir() -> PathBuf {
    Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf()
}

fn workspace_dir() -> PathBuf {
    package_dir().join("..")
}

async fn read_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(&path)
        .await
        .unwrap_or_else(|_| panic!("Could not find file, {:?}", path));
    let mut block = String::new();
    file.read_to_string(&mut block).await?;
    Ok(block)
}
