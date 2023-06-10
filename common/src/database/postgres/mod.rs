use lazy_regex::{regex, Lazy, Regex};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use crate::{database::Database, error::EmResult};

pub mod build;
pub mod connection;
pub mod listener;
pub mod test;

/// Postgresql implementation of the [Database] interface
pub struct Postgres;

impl Database for Postgres {
    type ConnectionOptions = PgConnectOptions;
    type ConnectionPool = PgPool;

    async fn create_pool(
        options: Self::ConnectionOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> EmResult<Self::ConnectionPool> {
        let pool = PgPoolOptions::new()
            .min_connections(min_connection)
            .max_connections(max_connections)
            .connect_with(options)
            .await?;
        Ok(pool)
    }

    fn create_pool_lazy(
        options: Self::ConnectionOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> Self::ConnectionPool {
        PgPoolOptions::new()
            .min_connections(min_connection)
            .max_connections(max_connections)
            .connect_lazy_with(options)
    }
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

/// Format the provided `block` so that is can be executed as an anonymous block of Postgresql
/// code
fn format_anonymous_block(block: &str) -> String {
    match block.split_whitespace().next() {
        Some("do") | None => block.to_owned(),
        Some("begin" | "declare") => format!("do $body$\n{block}\n$body$;"),
        Some(_) if TYPE_REGEX.is_match(block) => process_type_definition(block),
        Some(_) => format!("do $body$\nbegin\n{block}\nend;\n$body$;"),
    }
}
