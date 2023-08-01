use lazy_regex::{regex, Lazy, Regex};
use sqlx::PgPool;

use crate::{
    database::{
        postgres::{format_anonymous_block, Postgres},
        test::DatabaseTester,
        Database, RolledBackTransactionResult,
    },
    error::EmResult,
    package_dir, read_file,
};

static ENUM_REGEX: &Lazy<Regex, fn() -> Regex> = regex!(
    r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+)\s+as\s+enum\s*\((?P<labels>[^;]+)\s*\);"
);
static COMPOSITE_REGEX: &Lazy<Regex, fn() -> Regex> = regex!(
    r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+?)\s+as\s*\((?P<attributes>[^;]+)\);"
);

/// Check a database build unit to see if it defines an enum creation. If it does, it checks to see
/// if all it's specified labels can be found within the database's definition of the enum. If the
/// unit is not an enum, then the function exits immediately with an [Ok].
/// # Errors
/// This function will return an error if the contents of `file_name` do not match the `ENUM_REGEX`
/// pattern. Technically the function can also return an error if the captures of the regex pattern
/// are not found, but they must exist to match the pattern so that should never happen.
pub async fn check_for_enum(pool: &PgPool, file_name: &str) -> EmResult<()> {
    let file_path = package_dir()?.join("database").join(file_name);
    let block = read_file(file_path).await?;
    let Some(captures) = ENUM_REGEX.captures(&block) else {
        Err("Provided file does not match the ENUM_REGEX pattern")?
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
/// # Errors
/// This function will return an error if the contents of `file_name` do not match the
/// `COMPOSITE_REGEX` pattern. Technically the function can also return an error if the captures of
/// the regex pattern are not found, but they must exist to match the pattern so that should never
/// happen.
pub async fn check_for_composite(pool: &PgPool, file_name: &str) -> EmResult<()> {
    let file_path = package_dir()?.join("database").join(file_name);
    let block = read_file(file_path).await?;
    let Some(captures) = COMPOSITE_REGEX.captures(&block) else {
        return Ok(());
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

/// Run the specified `test_file` as an anonymous block within a rolled back transaction.
/// # Errors
/// This function will return an error if the package directory cannot be found, the `test_file`
/// cannot be read or an error is returned from the anonymous block execution (or rollback).
pub async fn run_db_test(pool: &PgPool, test_file: &str) -> EmResult<()> {
    let tester = PgDatabaseTester::create(pool);
    let test_path = package_dir()?.join("database/tests").join(test_file);
    let block = read_file(test_path).await?;
    let result = tester.execute_anonymous_block_transaction(&block).await;
    if let Some(block_error) = &result.block_error {
        Err(format!("{block_error}"))?
    }
    if let Some(transaction_error) = &result.block_error {
        Err(format!("{transaction_error}"))?
    }
    Ok(())
}

/// Postgresql implementation of a [DatabaseTester]
pub struct PgDatabaseTester {
    pool: PgPool,
}

impl DatabaseTester for PgDatabaseTester {
    type BlockError = sqlx::Error;
    type Database = Postgres;
    type TransactionError = sqlx::Error;

    fn create(pool: &<Self::Database as Database>::ConnectionPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn execute_anonymous_block_transaction(
        &self,
        block: &str,
    ) -> RolledBackTransactionResult<Self::BlockError, Self::TransactionError> {
        let block = format_anonymous_block(block);
        let mut result = RolledBackTransactionResult::default();
        let mut transaction = match self.pool.begin().await {
            Ok(inner) => inner,
            Err(error) => {
                result.with_transaction_error(error);
                return result;
            }
        };
        if let Err(error) = sqlx::query(&block).execute(&mut transaction).await {
            result.with_block_error(error);
        }
        if let Err(error) = transaction.rollback().await {
            result.with_transaction_error(error);
        }
        result
    }
}
