use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

fn get_relative_path(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut result = std::env::current_dir()?;
    result.push(path.trim_start_matches('/'));
    Ok(result)
}

#[derive(Debug, Deserialize)]
struct DbBuild {
    common_dependencies: Vec<String>,
    entries: Vec<DbBuildEntry>,
}

impl DbBuild {
    fn entries_ordered(&self) -> OrderIter<'_> {
        OrderIter::new(&self.entries)
    }
}

#[derive(Debug, Deserialize)]
struct DbBuildEntry {
    name: String,
    dependencies: Vec<String>,
}

impl DbBuildEntry {
    fn dependencies_met(&self, completed: &HashSet<String>) -> bool {
        self.dependencies.is_empty() || self.dependencies.iter().all(|d| completed.contains(d))
    }
}

struct OrderIter<'e> {
    entries: &'e Vec<DbBuildEntry>,
    returned: HashSet<usize>,
    completed: HashSet<String>,
}

impl<'e> OrderIter<'e> {
    fn new(entries: &'e Vec<DbBuildEntry>) -> Self {
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
                self.completed.insert(entry.name.clone());
                return Some(entry)
            }
        }
        if self.returned.len() != self.entries.len() {
            panic!("Exited iterator with remaining objects to create but not all dependencies resolved")
        }
        None
    }
}

async fn db_build(path: PathBuf) -> Result<(String, DbBuild), Box<dyn std::error::Error>> {
    let dir = path.parent().expect("Directory does not have a parent");
    let mut file = File::open(&path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let db_build: DbBuild = serde_json::from_str(&contents)?;
    Ok((dir.to_string_lossy().into_owned(), db_build))
}

async fn read_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)
        .await
        .unwrap_or_else(|_| panic!("Could not find file, {:?}", path));
    let mut block = String::new();
    file.read_to_string(&mut block).await?;
    Ok(block)
}

async fn execute_anonymous_block(block: String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let block = match block.split_whitespace().next() {
        Some("do") => block,
        Some("begin" | "declare") => format!("do $body$\n{}\n$body$;", block),
        Some(_) => format!("do $body$\nbegin\n{}\nend;\n$body$;", block),
        None => block,
    };
    sqlx::query(&block).execute(pool).await?;
    Ok(())
}

async fn build_common_schema(
    schema: &str,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_relative_path(&format!("common-database/{}/build.json", schema))?;
    let (directory, db_build) = db_build(path).await?;

    for entry in db_build.entries_ordered() {
        let block = read_file(&format!("{}/{}", directory, entry.name)).await?;
        if let Err(error) = execute_anonymous_block(block, pool).await {
            return Err(format!("Error running schema build {:?}. {}", entry.name, error).into());
        };
    }
    Ok(())
}

async fn build_schema(
    schema_build_path: &str,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_relative_path(&format!("{}/build.json", schema_build_path))?;
    let (directory, db_build) = db_build(path).await?;

    if !db_build.common_dependencies.is_empty() {
        for dep in &db_build.common_dependencies {
            build_common_schema(dep, pool).await?
        }
    }

    for entry in db_build.entries_ordered() {
        let block = read_file(&format!("{}/{}", directory, entry.name)).await?;
        if let Err(error) = execute_anonymous_block(block, pool).await {
            return Err(format!("Error running schema build {:?}. {}", entry.name, error).into());
        };
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = workflow_engine::create_we_db_pool().await?;
    build_schema("/workflow-engine/database", &pool).await?;
    Ok(())
}
