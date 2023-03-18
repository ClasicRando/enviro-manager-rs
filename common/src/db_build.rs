use serde::Deserialize;
use sqlx::PgPool;
use std::path::PathBuf;
use std::collections::HashSet;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::{execute_anonymous_block, package_dir, read_file, workspace_dir};

#[derive(Debug, Deserialize)]
pub(crate) struct DbBuild {
    pub(crate) common_dependencies: Vec<String>,
    pub(crate) entries: Vec<DbBuildEntry>,
}

impl DbBuild {
    fn entries_ordered(&self) -> OrderIter<'_> {
        OrderIter::new(&self.entries)
    }

    async fn run(
        &self,
        directory: &PathBuf,
        pool: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for dep in &self.common_dependencies {
            build_common_schema(dep, pool).await?
}

        for entry in self.entries_ordered() {
            entry.run(directory, pool).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct DbBuildEntry {
    pub(crate) name: String,
    dependencies: Vec<String>,
}

impl DbBuildEntry {
    fn dependencies_met<'e>(&self, completed: &'e HashSet<&'e str>) -> bool {
        self.dependencies.is_empty()
            || self
                .dependencies
                .iter()
                .all(|d| completed.contains(d.as_str()))
    }

    async fn run(
        &self,
        directory: &PathBuf,
        pool: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = directory.join(&self.name);
        let block = read_file(&path).await?;
        if let Err(error) = execute_anonymous_block(block, pool).await {
            return Err(format!("Error running schema build {:?}. {}", self.name, error).into());
        };
        Ok(())
    }
}

struct OrderIter<'e> {
    entries: &'e [DbBuildEntry],
    returned: HashSet<usize>,
    completed: HashSet<&'e str>,
}

impl<'e> OrderIter<'e> {
    fn new(entries: &'e [DbBuildEntry]) -> Self {
        Self {
            entries,
            returned: HashSet::new(),
            completed: HashSet::new(),
        }
    }
}

impl<'e> Iterator for OrderIter<'e> {
    type Item = &'e DbBuildEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries.is_empty() {
            return None;
        }
        for (i, entry) in self.entries.iter().enumerate() {
            if self.returned.contains(&i) {
                continue;
            }
            if entry.dependencies_met(&self.completed) {
                self.returned.insert(i);
                self.completed.insert(&entry.name);
                return Some(entry);
            }
        }
        if self.returned.len() != self.entries.len() {
            panic!("Exited iterator with remaining objects to create but not all dependencies resolved")
        }
        None
    }
}

pub(crate) async fn db_build(directory: &PathBuf) -> Result<DbBuild, Box<dyn std::error::Error>> {
    let path = directory.join("build.json");
    let mut file = File::open(&path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let db_build: DbBuild = serde_json::from_str(&contents)?;
    Ok(db_build)
}

async fn build_common_schema(
    schema: &str,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema_directory = workspace_dir().join("common-database").join(schema);
    let db_build = db_build(&schema_directory).await?;

    for entry in db_build.entries_ordered() {
        entry.run(&schema_directory, pool).await?;
    }
    Ok(())
}

pub async fn build_schema(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let database_directory = package_dir().join("database");
    let db_build = db_build(&database_directory).await?;

    db_build.run(&database_directory, pool).await?;
    Ok(())
}
