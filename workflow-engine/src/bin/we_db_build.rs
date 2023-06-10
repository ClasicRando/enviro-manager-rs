use common::database::{build::build_database, postgres::build::PgDatabaseBuilder};
use log::error;
use workflow_engine::database::db_options;

#[tokio::main]
async fn main() {
    let options = match db_options() {
        Ok(inner) => inner,
        Err(error) => {
            error!("Error fetching database options. {error}");
            return;
        }
    };
    build_database::<PgDatabaseBuilder, _, _>("workflow-engine/we_db_build_log.yml", options).await
}
