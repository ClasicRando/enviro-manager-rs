use log::{error, info, warn};
use sqlx::PgPool;

use crate::{
    database::{
        build::{DatabaseBuilder, DbBuild},
        postgres::{format_anonymous_block, Postgres},
    },
    error::EmResult,
    package_dir, read_file,
};

/// Postgresql implementation of a [DatabaseBuilder]
pub struct PgDatabaseBuilder {
    /// Shared pool of postgresql connections
    pool: PgPool,
}

impl DatabaseBuilder for PgDatabaseBuilder {
    type Database = Postgres;

    fn create(pool: PgPool) -> Self {
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
            error!("Error building {} database. {}", database_target, error);
            return;
        }

        if db_refresh {
            let scaffold_path =
                database_directory.join(format!("{database_target}_scaffold.pgsql"));

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
        let schema_names_option: Option<String> = sqlx::query_scalar(
            r#"
            select string_agg(schema_name,', ')
            from information_schema.schemata
            where schema_owner = current_user"#,
        )
        .fetch_one(&self.pool)
        .await?;
        let Some(schema_names) = schema_names_option else {
            warn!("Current user does not own any schemas");
            return Ok(())
        };

        sqlx::query(&format!("drop schema if exists {schema_names} cascade"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn execute_anonymous_block(&self, block: &str) -> EmResult<()> {
        let block = format_anonymous_block(block);
        sqlx::query(&block).execute(&self.pool).await?;
        Ok(())
    }
}
