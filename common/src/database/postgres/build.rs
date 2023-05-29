use std::path::PathBuf;

use lazy_regex::{regex, Lazy, Regex};
use log::{error, info, warn};
use sqlx::{PgPool, Pool, Postgres};

use crate::{
    database::build::{DatabaseBuilder, DbBuild},
    error::EmResult,
    package_dir, read_file,
};

/// Regex to find and parse a create type postgres statement
static TYPE_REGEX: &Lazy<Regex, fn() -> Regex> =
    regex!(r"^create\s+type\s+(?P<schema>[^.]+)\.(?P<name>[^.]+)\s+as(?P<definition>[^;]+);");

///
pub struct PgDatabaseBuilder {
    ///
    pool: PgPool,
}

impl PgDatabaseBuilder {
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
            Some(_) if TYPE_REGEX.is_match(block) => Self::process_type_definition(block),
            Some(_) => format!("do $body$\nbegin\n{block}\nend;\n$body$;"),
        }
    }
}

impl DatabaseBuilder for PgDatabaseBuilder {
    type Database = Postgres;

    fn create(pool: Pool<Self::Database>) -> Self {
        Self { pool }
    }

    async fn build_database(&self) {
        let database_result = sqlx::query_scalar("select current_database()")
            .fetch_one(&self.pool)
            .await;
        let database_target: String = match database_result {
            Ok(inner) => inner,
            Err(error) => {
                error!("Could not fetch the current database. {error}");
                return;
            }
        };
        let db_refresh = std::env::var("DB_REFRESH")
            .map(|v| &v == "true")
            .unwrap_or_default();

        let Some(name) = std::env::var("DB_BUILD_TARGET").ok() else {
            warn!(
            "Could not find a value for the database build target. Please specify with the \
             'DB_BUILD_TARGET' environment variable"
        );
            return;
        };

        info!("Target specified as '{database_target}' to rebuild");

        if db_refresh {
            info!("Refresh specified for the '{database_target}' database");
            if let Err(error) = self.refresh_database().await {
                error!("Error refreshing '{database_target}' database. {error}");
                return;
            }
        }

        let database_directory = match package_dir().map(|d| d.join("database")) {
            Ok(inner) => inner,
            Err(error) => {
                error!("Package directory could not be found. {error}");
                return;
            }
        };
        let db_build = match DbBuild::new(&database_directory).await {
            Ok(inner) => inner,
            Err(error) => {
                error!("Error creating DbBuild instance. {error}");
                return;
            }
        };
        if let Err(error) = db_build.run(&database_directory, self).await {
            error!("Error building {} database. {}", name, error);
            return;
        }

        if db_refresh {
            let scaffold_path =
                PathBuf::from(format!("./users/database/{database_target}_scaffold.pgsql"));

            if !scaffold_path.exists() {
                info!("No scaffold file. Continuing");
                return;
            }

            let block_result = match read_file(&scaffold_path).await {
                Ok(sql_block) => self.execute_anonymous_block(&sql_block).await,
                Err(error) => {
                    error!("Error reading the database scaffold file. {}", error);
                    return;
                }
            };
            if let Err(error) = block_result {
                error!("Error scaffolding test database. {}", error)
            }
        }
    }

    async fn refresh_database(&self) -> EmResult<()> {
        let schema_names: Vec<String> = sqlx::query_scalar(
            r#"
            select schema_name
            from information_schema.schemata
            where schema_owner = current_user"#,
        )
        .fetch_all(&self.pool)
        .await?;
        for schema_name in schema_names {
            sqlx::query(&format!("drop schema if exists {schema_name}"))
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn execute_anonymous_block(&self, block: &str) -> EmResult<()> {
        let block = Self::format_anonymous_block(block);
        sqlx::query(&block).execute(&self.pool).await?;
        Ok(())
    }
}
