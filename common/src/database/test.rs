use std::path::Path;

use lazy_regex::{regex, Lazy, Regex};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::{
    fs::{read_dir, File},
    io::AsyncReadExt,
};

use crate::{
    database::build::DbBuild, execute_anonymous_block, execute_anonymous_block_transaction,
    package_dir, read_file, workspace_dir,
};

/// Represents a single entry in the list of test directory entries. Deserialized from a
/// `test.json` found within a test directory. `name` points to the file to read & run and
/// `rollback` tells the test runner to execute the file within a rolled back transaction.
#[derive(Deserialize)]
struct TestListEntry {
    /// Name of the test script file to run
    name: String,
    /// Flag indicating if the test should be executed in a rolled back transaction
    rollback: bool,
}

impl TestListEntry {
    /// Run the test entry represented by the [TestListEntry] instance. All errors that may arise
    /// within the function call get stored in the `results` buffer.
    async fn run_test(&self, directory: &Path, pool: &PgPool, results: &mut Vec<String>) {
        let path = directory.join(&self.name);
        let block = match read_file(&path).await {
            Ok(inner) => inner,
            Err(error) => {
                results.push(format!("Failed reading test file {:?}\n{}", path, error));
                return;
            }
        };

        if self.rollback {
            let result = execute_anonymous_block_transaction(&block, pool).await;
            result.add_to_error_list(&path, results);
            return;
        }

        let result = execute_anonymous_block(&block, pool).await;
        if let Err(error) = result {
            results.push(format!("Failed running test in {:?}\n{}", path, error))
        }
    }
}

/// Read the list of tests that is contained within the `test_directory`. Returns the test names as
/// a vector of [Path].
async fn read_tests_list(
    test_directory: &Path,
) -> Result<Vec<TestListEntry>, Box<dyn std::error::Error>> {
    let tests_file = test_directory.join("tests.json");
    let mut file = File::open(&tests_file).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let test_list: Vec<TestListEntry> = serde_json::from_str(&contents)?;
    Ok(test_list)
}

/// Run the tests found within the `tests_path` directory. Parses the provided 'tests.txt' file
/// found within the directory and runs the prescribed tests.
async fn run_test_directory(
    tests_path: &Path,
    pool: &PgPool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let tests = read_tests_list(tests_path).await?;
    for entry in tests {
        entry.run_test(tests_path, pool, &mut results).await
    }
    Ok(results)
}

/// Runs the tests found within the main `tests_path` directory provided. Reads every entry in the
/// directory, handling files as standalone tests that are executed and directories as test
/// directories that contain a 'tests.txt' file with the required tests within the directory.
///
/// Every test is run unless an error outside the tests is raised. All test results are packed into
/// a result vector and the vector is checked for contents at the end to show all failing tests.
async fn run_tests(tests_path: &Path, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    if !tests_path.exists() {
        return Ok(());
    }

    let mut results = Vec::new();
    let mut entries = read_dir(tests_path).await?;
    while let Some(file) = entries.next_entry().await? {
        let file_path = file.path();
        if file.file_type().await?.is_dir() {
            let mut result = run_test_directory(&file_path, pool).await?;
            results.append(&mut result);
            continue;
        }
        let block = read_file(&file_path).await?;
        let result = execute_anonymous_block(&block, pool).await;
        if let Err(error) = result {
            results.push(format!("Failed running test in {:?}\n{}", file_path, error))
        }
    }
    assert!(
        results.is_empty(),
        "Failed database tests\n{}",
        results.join("\n")
    );
    Ok(())
}

/// Run all tests for a common database schema. See [run_tests] for details on execution.
async fn run_common_db_tests(
    pool: &PgPool,
    common_db_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tests = workspace_dir()?
        .join("common-database")
        .join(common_db_name)
        .join("tests");
    run_tests(&tests, pool).await
}

static ENUM_REGEX: &Lazy<Regex, fn() -> Regex> = regex!(
    r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+)\s+as\s+enum\s*\((?P<labels>[^;]+)\s*\);"
);
static COMPOSITE_REGEX: &Lazy<Regex, fn() -> Regex> = regex!(
    r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+?)\s+as\s*\((?P<attributes>[^;]+)\);"
);

/// Check a database build unit to see if it defines an enum creation. If it does, it checks to see
/// if all it's specified labels can be found within the database's definition of the enum. If the
/// unit is not an enum, then the function exits immediately with an [Ok].
async fn check_for_enum(block: &str, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let Some(captures) = ENUM_REGEX.captures(block) else {
        return Ok(())
    };
    let Some(schema) = captures.name("schema") else {
        Err("No 'schema' capture group present in enum definition")?
    };
    let Some(name) = captures.name("name") else {
        Err("No 'name' capture group present in enum definition")?
    };
    let Some(labels) = captures.name("labels") else {
        Err("No 'labels' capture group present in enum definition")?
    };
    let labels: Vec<&str> = labels
        .as_str()
        .split(',')
        .map(|label| label.trim().trim_matches('\''))
        .collect();
    sqlx::query("call data_check.check_enum_definition($1,$2,$3)")
        .bind(schema.as_str())
        .bind(name.as_str())
        .bind(labels)
        .execute(pool)
        .await?;
    Ok(())
}

/// Check a database build unit to see if it defines a composite creation. If it does, it checks to
/// see if all it's specified attributes can be found within the database's definition of the
/// composite. If the unit is not a composite, then the function exits immediately with an [Ok].
async fn check_for_composite(block: &str, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let Some(captures) = COMPOSITE_REGEX.captures(block) else {
        return Ok(())
    };
    let Some(schema) = captures.name("schema") else {
        Err("No 'schema' capture group present in composite definition")?
    };
    let Some(name) = captures.name("name") else {
        Err("No 'name' capture group present in composite definition")?
    };
    let Some(attributes) = captures.name("attributes") else {
        Err("No 'attributes' capture group present in composite definition")?
    };
    let attributes: Vec<&str> = attributes
        .as_str()
        .split(',')
        .map(|label| label.trim())
        .collect();
    sqlx::query("call data_check.check_composite_definition($1,$2,$3)")
        .bind(schema.as_str())
        .bind(name.as_str())
        .bind(attributes)
        .execute(pool)
        .await?;
    Ok(())
}

/// Run all database tests for the current package against the provided connection `pool`.
///
/// The database is automatically updated where possible using the [build_database] command and if
/// the database directory of the package contains a 'test_data.pgsql' file, that is also run to
/// refresh the test database.
///
/// For common database dependencies that exist within the database's 'build.json' file, all tests
/// are run as well to ensure behaviour works as intended.
///
/// Outside of the tests provided within the `tests` sub directory of `database`, all enum and
/// composite creation units that match the expected format are also checked since a change in
/// either type definition might not make it to the database and since Postgresql does not support
/// any replace or update DDL statements for either type definition. In those cases a full refresh
/// of the database might be required or manual alter statements must be created.
/// # Errors
/// TODO
pub async fn run_db_tests(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let package_dir = package_dir()?;

    let test_refresh_script = package_dir.join("database").join("test_data.pgsql");
    if test_refresh_script.exists() {
        let block = read_file(&test_refresh_script).await?;
        execute_anonymous_block(&block, pool).await?;
    }

    let schema_directory = package_dir.join("database");
    let db_build = DbBuild::new(&schema_directory).await?;

    for common_schema in &db_build.common_dependencies {
        run_common_db_tests(pool, common_schema).await?;
    }

    for entry in db_build.entries {
        let block = read_file(&schema_directory.join(&entry.name)).await?;
        check_for_enum(&block, pool).await?;
        check_for_composite(&block, pool).await?;
    }

    let tests = schema_directory.join("tests");
    run_tests(&tests, pool).await
}
