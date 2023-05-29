use std::{path::PathBuf, str::FromStr};

use common::{
    database::{
        build::build_database,
        connection::{ConnectionBuilder, PgConnectionBuilder},
    },
    error::{EmError, EmResult},
};
use log::{error, info, warn};
use sqlx::{postgres::PgConnectOptions, PgPool};
use users::database::{db_options, test_db_options};

pub enum DatabaseTarget {
    Production,
    Test,
}

impl DatabaseTarget {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Production => "prod",
            Self::Test => "test",
        }
    }
}

impl FromStr for DatabaseTarget {
    type Err = EmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "test" => Ok(Self::Test),
            "prod" => Ok(Self::Production),
            _ => Err(EmError::Generic(format!(
                "Could not parse the database target. Found '{s}'"
            ))),
        }
    }
}

/// Clear every user schema within the database specified by the `pool` that is owned by the
/// `sql_user` provided.
async fn refresh_database(pool: &PgPool, sql_user: &str) -> EmResult<()> {
    let schema_names: Vec<String> = sqlx::query_scalar(
        r#"
        select schema_name
        from information_schema.schemata
        where schema_owner = $1"#,
    )
    .bind(sql_user)
    .fetch_all(pool)
    .await?;
    for schema_name in schema_names {
        sqlx::query(&format!("drop schema if exists {schema_name}"))
            .execute(pool)
            .await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    log4rs::init_file("users/users_db_build_log.yml", Default::default()).unwrap();
    let db_refresh = std::env::var("USERS_DB_REFRESH")
        .map(|v| &v == "true")
        .unwrap_or_default();

    let Some(name) = std::env::var("USERS_DB_BUILD_TARGET").ok() else {
        warn!(
            "Could not find a value for the database build target. Please specify with the \
             'USERS_DB_BUILD_TARGET' environment variable"
        );
        return;
    };
    let database_target = match DatabaseTarget::from_str(&name) {
        Ok(inner) => inner,
        Err(error) => {
            warn!("{error}");
            return;
        }
    };

    info!(
        "Target specified as '{}' to rebuild",
        database_target.as_str()
    );
    let options = match database_target {
        DatabaseTarget::Test => match test_db_options() {
            Ok(inner) => inner,
            Err(error) => {
                error!("Error getting test database options. {error}");
                return;
            }
        },
        DatabaseTarget::Production => match db_options() {
            Ok(inner) => inner,
            Err(error) => {
                error!("Error getting prod database options. {error}");
                return;
            }
        },
    };
    let pool = match PgConnectionBuilder::create_pool(options, 1, 1).await {
        Ok(inner) => inner,
        Err(error) => {
            error!("Error getting database connection pool. {error}");
            return;
        }
    };

    if db_refresh {
        info!(
            "Refresh specified for the '{}' database",
            database_target.as_str()
        );
        let sql_user = match database_target {
            DatabaseTarget::Production => "users_admin",
            DatabaseTarget::Test => "users_test",
        };
        match refresh_database(&pool, sql_user).await {
            Ok(pool) => Ok(pool),
            Err(error) => {
                error!(
                    "Error refreshing '{}' database. {error}",
                    database_target.as_str()
                );
                return;
            }
        }
    }

    if let Err(error) = build_database(&pool).await {
        error!("Error building {} database. {}", name, error);
        return;
    }

    if db_refresh {
        let scaffold_path = PathBuf::from(format!(
            "./users/database/{}_scaffold.pgsql",
            database_target.as_str()
        ));

        if !scaffold_path.exists() {
            info!("No scaffold file. Continuing");
            return;
        }

        let block_result = match common::read_file(&scaffold_path).await {
            Ok(sql_block) => common::execute_anonymous_block(&sql_block, &pool).await,
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
