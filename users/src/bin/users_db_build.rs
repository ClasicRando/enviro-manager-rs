use common::database::{
    build::build_database,
    postgres::{build::PgDatabaseBuilder, connection::PgConnectionBuilder},
};
use log::error;
use users::database::db_options;

#[tokio::main]
async fn main() {
    let options = match db_options() {
        Ok(inner) => inner,
        Err(error) => {
            error!("Error fetching database options. {error}");
            return;
        }
    };
    build_database::<PgDatabaseBuilder, PgConnectionBuilder, _, _>(
        "users/users_db_build_log.yml",
        options,
    )
    .await
}
