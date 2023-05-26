#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![warn(clippy::cloned_instead_of_copied)]
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::create_dir)]
#![warn(clippy::empty_structs_with_brackets)]
#![warn(clippy::equatable_if_let)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::expect_used)]
#![warn(clippy::fn_params_excessive_bools)]
#![warn(clippy::from_iter_instead_of_collect)]
#![warn(clippy::future_not_send)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::inefficient_to_string)]
#![warn(clippy::manual_let_else)]
#![warn(clippy::manual_string_new)]
#![warn(clippy::match_on_vec_items)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::missing_assert_message)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::needless_collect)]
#![warn(clippy::needless_continue)]
#![warn(clippy::needless_for_each)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_if_let_else)]
#![warn(clippy::panic)]
#![warn(clippy::partial_pub_fields)]
#![warn(clippy::print_stdout)]
#![warn(clippy::pub_use)]
#![warn(clippy::string_to_string)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_slice)]
#![warn(clippy::too_many_arguments)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::unnecessary_box_returns)]
#![warn(clippy::unused_async)]
#![warn(clippy::unused_self)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::use_self)]
#![warn(clippy::wildcard_imports)]

//! Common components of the EnivroManager application suite

use std::path::{Path, PathBuf};

use lazy_regex::{regex, Lazy, Regex};
use sqlx::PgPool;
use tokio::{fs::File, io::AsyncReadExt};

use crate::error::EmResult;

pub mod api;
pub mod database;
pub mod error;

/// Returns a [PathBuf] pointing to the directory of the current package. Utilizes the
/// 'CARGO_MANIFEST_DIR' cargo environment variable.
/// # Errors
/// This function will return an error if the `CARGO_MANIFEST_DIR` environment variable is not set
pub fn package_dir() -> EmResult<PathBuf> {
    Ok(Path::new(&std::env::var("CARGO_MANIFEST_DIR")?).to_path_buf())
}

/// Returns a [PathBuf] pointing to the current workspace. Fetches the package directory using
/// [package_dir] then navigates to the parent directory to find the workspace.
/// # Errors
/// This function will return an error if the `CARGO_MANIFEST_DIR` environment variable is not set
fn workspace_dir() -> EmResult<PathBuf> {
    Ok(package_dir()?.join(".."))
}

/// Read the specified file using the `path` provided, returning the contents as a single [String]
/// buffer.
/// # Errors
/// This function will return an error if the file could not be opened or the contents of the file
/// could not be read into a [String] buffer.
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

/// Regex to find and parse a create type postgres statement
static TYPE_REGEX: &Lazy<Regex, fn() -> Regex> =
    regex!(r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+)\s+as(?P<definition>[^;]+);");

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
    format!("do $body$\nbegin\n{block}\nend;\n$body$;")
}

/// Format the provided `block` so that is can be executed as an anonymous block of Postgresql code
fn format_anonymous_block(block: &str) -> String {
    match block.split_whitespace().next() {
        Some("do") | None => block.to_owned(),
        Some("begin" | "declare") => format!("do $body$\n{block}\n$body$;"),
        Some(_) if TYPE_REGEX.is_match(block) => process_type_definition(block),
        Some(_) => format!("do $body$\nbegin\n{block}\nend;\n$body$;"),
    }
}

/// Execute the provided `block` of Postgresql code against the `pool`. If the block does not match
/// the required formatting to be an anonymous block, the code is wrapped in the required code to
/// ensure the execution can be completed.
/// # Errors
/// This function will return an error if executing the SQL query `block` returns an error from the
/// database.
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
            error_list.push(format!("Failed running test in {path:?}\n{error}"))
        }
        if let Some(error) = self.transaction_error {
            error_list.push(format!(
                "Failed during rollback of test in {path:?}\n{error}"
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
