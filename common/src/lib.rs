use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;

use std::path::{Path, PathBuf};

use tokio::{fs::File, io::AsyncReadExt};

pub mod db_build;
pub mod db_test;

fn package_dir() -> PathBuf {
    Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf()
}

fn workspace_dir() -> PathBuf {
    package_dir().join("..")
}

async fn read_file(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)
        .await
        .unwrap_or_else(|_| panic!("Could not find file, {:?}", path));
    let mut block = String::new();
    file.read_to_string(&mut block).await?;
    Ok(block)
}

lazy_static! {
    static ref TYPE_REGEX: Regex =
        Regex::new(r"^create type (?P<schema>[^.]+)\.(?P<name>[^.]+) as(?P<definition>[^;]+);")
            .unwrap();
}

fn process_type_definition(block: String) -> String {
    let block = TYPE_REGEX.replace(
        &block,
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

async fn execute_anonymous_block(block: String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let block = match block.split_whitespace().next() {
        Some("do") => block,
        Some("begin" | "declare") => format!("do $body$\n{}\n$body$;", block),
        Some(_) if TYPE_REGEX.is_match(&block) => process_type_definition(block),
        Some(_) => format!("do $body$\nbegin\n{}\nend;\n$body$;", block),
        None => block,
    };
    sqlx::query(&block).execute(pool).await?;
    Ok(())
}
