use std::{collections::HashSet, path::Path};

use log::error;
use serde::Deserialize;

use crate::{
    database::Database,
    error::EmResult,
    fs::{read_file, workspace_dir},
};

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
    /// Extract a [DbBuild] instance using the `directory` provided. The `directory` should point to
    /// a directory that contains a "build.json" file that can be deserializable into the [DbBuild]
    /// struct.
    pub(crate) async fn new<P: AsRef<Path> + Send>(path: P) -> EmResult<Self> {
        let path = path.as_ref().join("build.json");
        let contents = read_file(path).await?;
        let db_build: Self = serde_json::from_str(&contents)?;
        Ok(db_build)
    }

    /// Returns the `entries` wrapped in a custom [Iterator] that orders the results by the next
    /// available entry that can be built. This ensures that an entry is only built once all
    /// dependencies are met.
    fn entries_ordered(&self) -> OrderIter<'_> {
        OrderIter::new(&self.entries)
    }

    /// Run the database build operations by building the common schema requirements then
    /// proceeding to run each [DbBuildEntry] to completion.
    pub(crate) async fn run<B, P>(&self, directory: P, builder: &B) -> EmResult<()>
    where
        B: DatabaseBuilder,
        P: AsRef<Path> + Send + Sync,
    {
        for dep in &self.common_dependencies {
            build_common_schema(dep, builder).await?
        }

        for entry in self.entries_ordered() {
            entry.run(directory.as_ref(), builder).await?;
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
    async fn run<B>(&self, directory: &Path, builder: &B) -> EmResult<()>
    where
        B: DatabaseBuilder,
    {
        let path = directory.join(&self.name);
        let block = read_file(&path).await?;
        if let Err(error) = builder.execute_anonymous_block(&block).await {
            return Err(format!("Error running schema build {:?}. {error}", self.name).into());
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
            log::error!(
                "Exited iterator with remaining objects to create but not all dependencies \
                 resolved"
            )
        }
        None
    }
}

/// Build the common `schema` by name. Extracts a [DbBuild] instance from the specified `schema`
/// directory, building each entry in order as required by dependency hierarchy.
async fn build_common_schema<B>(schema: &str, builder: &B) -> EmResult<()>
where
    B: DatabaseBuilder,
{
    let schema_directory = workspace_dir()?.join("common-database").join(schema);
    let db_build = DbBuild::new(&schema_directory).await?;

    for entry in db_build.entries_ordered() {
        entry.run(&schema_directory, builder).await?;
    }
    Ok(())
}

/// Behaviour to allow for database to be populated with all the required objects. This type should
/// be implemented for every [Database] implementation once and only utilized as a generic
/// parameter to [build_database].
pub trait DatabaseBuilder
where
    Self: Send + Sync,
{
    /// Database variation that will store the database objects
    type Database: Database;
    /// Create a new instance of the [DatabaseBuilder] using the [Database]'s connection pool type
    fn create(pool: <Self::Database as Database>::ConnectionPool) -> Self;
    /// Build the database. This operation is intended to be executed against a populated database
    /// so the scripts should account for existing objects that cannot be replaced with new
    /// versions. Deployments for objects that need to be altered should be handled manually... for
    /// now. Any errors that would otherwise be returned during this stage should be handled as if
    /// this function is a terminal operations of a script (i.e. log and exit early).
    async fn build_database(&self);
    /// Refresh the current database to a clean instance with no entities existing.
    /// # Errors
    /// This function will return an error if the refresh actions returns database errors.
    async fn refresh_database(&self) -> EmResult<()>;
    /// Execute the provided `block` of SQL code against the [DatabaseBuilder]. If the block does
    /// not match the required formatting to be an anonymous block, the code is wrapped in the
    /// required code to ensure the execution can be completed.
    /// # Errors
    /// This function will return an error if executing the SQL query `block` returns an error from
    /// the database.
    async fn execute_anonymous_block(&self, block: &str) -> EmResult<()>;
}

/// Execute a build against the database specified by the connection `options` provided. All
/// messages will be logged using the configuration specified by `log_config_path`.
pub async fn build_database<B, D>(options: D::ConnectionOptions)
where
    B: DatabaseBuilder<Database = D>,
    D: Database,
{
    let pool = match D::create_pool(options, 1, 1).await {
        Ok(inner) => inner,
        Err(error) => {
            error!("Could not create a connection pool for database building. {error}");
            return;
        }
    };
    let builder = B::create(pool);
    builder.build_database().await
}
