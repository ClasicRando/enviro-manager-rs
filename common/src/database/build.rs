use std::{collections::HashSet, path::Path};

use serde::Deserialize;
use sqlx::PgPool;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{execute_anonymous_block, package_dir, read_file, workspace_dir};

/// Database builder object defining the common database dependencies and the schema entries
/// required.
///
/// Common dependencies are found within the `common-database` folder within the root workspace and
/// each entry in the vector specifies a name of the common schema required.
///
/// Entries are files within the package's database directory (or sub directories). Files can be a
/// single object, multiple linked objects (e.g. a table and it's indexes) or a standalone script
/// that must run.
#[derive(Debug, Deserialize)]
pub(crate) struct DbBuild {
    /// Common database schema dependencies as the names of the common schemas
    pub(crate) common_dependencies: Vec<String>,
    /// List of [DbBuildEntry] items for building the database
    pub(crate) entries: Vec<DbBuildEntry>,
}

impl DbBuild {
    /// Returns the `entries` wrapped in a custom [Iterator] that orders the results by the next
    /// available entry that can be built. This ensures that an entry is only built once all
    /// dependencies are met.
    fn entries_ordered(&self) -> OrderIter<'_> {
        OrderIter::new(&self.entries)
    }

    /// Run the database build operations by building the common schema requirements then
    /// proceeding to run each [DbBuildEntry] to completion.
    async fn run(&self, directory: &Path, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
        for dep in &self.common_dependencies {
            build_common_schema(dep, pool).await?
        }

        for entry in self.entries_ordered() {
            entry.run(directory, pool).await?;
        }
        Ok(())
    }
}

/// Database build entry specifying the name of the build unit contained with the `database`
/// directory of the current package as well as any other required units that must have already
/// been created.
#[derive(Debug, Deserialize)]
pub(crate) struct DbBuildEntry {
    /// Name of the build entry
    pub(crate) name: String,
    /// List of build entry items that are required before creating this entry
    dependencies: Vec<String>,
}

impl DbBuildEntry {
    /// Returns true if the `completed` units provided contain all the required dependencies of
    /// the build entry.
    fn dependencies_met<'e>(&self, completed: &'e HashSet<&'e str>) -> bool {
        self.dependencies.is_empty()
            || self
            .dependencies
            .iter()
            .all(|d| completed.contains(d.as_str()))
    }

    /// Run the build entry by fetching the entries file contents (relative path to the
    /// `directory` passed) and executing the entry's contents as an anonymous block against the
    /// `pool`.
    async fn run(&self, directory: &Path, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let path = directory.join(&self.name);
        let block = read_file(&path).await?;
        if let Err(error) = execute_anonymous_block(&block, pool).await {
            return Err(format!("Error running schema build {:?}. {}", self.name, error).into());
        };
        Ok(())
    }
}

/// Ordered [Iterator] providing the build entries in order of when units can be created/executed.
///
/// Contains the original vector of entries to be created as well as the name and indexes of the
/// completed entries.
struct OrderIter<'e> {
    /// Slice of [DbBuildEntry] items that this [Iterator] will will yield
    entries: &'e [DbBuildEntry],
    /// Indexes of [DbBuildEntry] items that have already been returned
    returned: HashSet<usize>,
    /// Names of the complete [DbBuildEntry] items
    completed: HashSet<&'e str>,
}

impl<'e> OrderIter<'e> {
    /// Create a new [OrderIter] with build `entries` provided.
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
            panic!(
                "Exited iterator with remaining objects to create but not all dependencies \
                 resolved"
            )
        }
        None
    }
}

/// Extract a [DbBuild] instance using the `directory` provided. The `directory` should point to a
/// directory that contains a "build.json" file that can be deserializable into the [DbBuild]
/// struct.
pub(crate) async fn db_build(directory: &Path) -> Result<DbBuild, Box<dyn std::error::Error>> {
    let path = directory.join("build.json");
    let mut file = File::open(&path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let db_build: DbBuild = serde_json::from_str(&contents)?;
    Ok(db_build)
}

/// Build the common `schema` by name. Extracts a [DbBuild] instance from the specified `schema`
/// directory, building each entry in order as required by dependency hierarchy.
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

/// Build the database as specified by the `database` directory of the current package. Build order
/// and units are found using the 'build.json' file in the `database` directory. See [DbBuild] for
/// expected JSON structure.
pub async fn build_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let database_directory = package_dir().join("database");
    let db_build = db_build(&database_directory).await?;

    db_build.run(&database_directory, pool).await?;
    Ok(())
}
