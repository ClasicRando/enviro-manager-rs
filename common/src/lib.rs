#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::missing_const_for_fn)]

//! Common components of the EnivroManager application suite

use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;
use tokio::{fs::File, io::AsyncReadExt};

use crate::error::EmResult;

pub mod api;
pub mod database;
pub mod error;

/// Returns a [PathBuf] pointing to the directory of the current package. Utilizes the
/// 'CARGO_MANIFEST_DIR' cargo environment variable.
pub fn package_dir() -> PathBuf {
    Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf()
}

/// Returns a [PathBuf] pointing to the current workspace. Fetches the package directory using
/// [package_dir] then navigates to the parent directory to find the workspace.
fn workspace_dir() -> PathBuf {
    package_dir().join("..")
}

/// Read the specified file using the `path` provided, returning the contents as a single [String]
/// buffer.
pub async fn read_file(path: impl AsRef<Path>) -> EmResult<String> {
    let path = path.as_ref();
    let mut file = match File::open(path).await {
        Ok(inner) => inner,
        Err(error) => return Err(format!("Could not open file, {:?}. {}", path, error).into()),
    };
    let mut block = String::new();
    file.read_to_string(&mut block).await?;
    Ok(block)
}

lazy_static! {
    ///
    static ref TYPE_REGEX: Regex = Regex::new(
        r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+)\s+as(?P<definition>[^;]+);"
    )
    .unwrap();
}

/// Process a Postgresql type definition `block`, updating the contents to not run the create
/// statement if the type already exists and wrapping the entire block as an anonymous block.
fn process_type_definition(block: &str) -> String {
    let block = TYPE_REGEX.replace(
        block,
        r#"
        if not exists(
            select 1
            from pg_namespace n
            join pg_type t on n.oid = t.typnamespace
            where
                n.nspname = '$schema'
                and t.typname = '$name'
        ) then
            create type ${schema}.$name as $definition;
        end if;
        "#,
    );
    format!("do $body$\nbegin\n{}\nend;\n$body$;", block)
}

/// Format the provided `block` so that is can be executed as an anonymous block of Postgresql code
fn format_anonymous_block(block: &str) -> String {
    match block.split_whitespace().next() {
        Some("do") | None => block.to_string(),
        Some("begin" | "declare") => format!("do $body$\n{}\n$body$;", block),
        Some(_) if TYPE_REGEX.is_match(block) => process_type_definition(block),
        Some(_) => format!("do $body$\nbegin\n{}\nend;\n$body$;", block),
    }
}

/// Execute the provided `block` of Postgresql code against the `pool`. If the block does not match
/// the required formatting to be an anonymous block, the code is wrapped in the required code to
/// ensure the execution can be completed.
pub async fn execute_anonymous_block(block: &str, pool: &PgPool) -> EmResult<()> {
    let block = format_anonymous_block(block);
    sqlx::query(&block).execute(pool).await?;
    Ok(())
}

/// Container for multiple optional errors that could arise from an execution of an anonymous block
/// of sql code with a rolled back transaction.
#[derive(Default)]
pub struct RolledBackTransactionResult {
    /// Error within the original sql block executed
    block_error: Option<sqlx::Error>,
    /// Error during the transaction rollback
    transaction_error: Option<sqlx::Error>,
}

impl RolledBackTransactionResult {
    /// Add or replace the current `block_error` attribute with `error`
    fn with_block_error(&mut self, error: sqlx::Error) {
        self.block_error = Some(error);
    }

    /// Add or replace the current `transaction_error` attribute with `error`
    fn with_transaction_error(&mut self, error: sqlx::Error) {
        self.transaction_error = Some(error);
    }

    /// Add errors within the result instance to the provided `error_list`, formatted with the test
    /// `path` in the message.
    pub(crate) fn add_to_error_list(self, path: &PathBuf, error_list: &mut Vec<String>) {
        if let Some(error) = self.block_error {
            error_list.push(format!("Failed running test in {:?}\n{}", path, error))
        }
        if let Some(error) = self.transaction_error {
            error_list.push(format!(
                "Failed during rollback of test in {:?}\n{}",
                path, error
            ))
        }
    }
}

/// Execute the provided `block` of Postgresql code against the `pool`. If the block does not match
/// the required formatting to be an anonymous block, the code is wrapped in the required code to
/// ensure the execution can be completed. The entire block is executed within a rolled back
/// transaction, returning the errors of the block and transaction rollback, if any, respectively
/// within a tuple.
async fn execute_anonymous_block_transaction(
    block: &str,
    pool: &PgPool,
) -> RolledBackTransactionResult {
    let block = format_anonymous_block(block);
    let mut result = RolledBackTransactionResult::default();
    let mut transaction = match pool.begin().await {
        Ok(inner) => inner,
        Err(error) => {
            result.with_block_error(error);
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
