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
    if let Err(error) = log4rs::init_file("workflow-engine/we_db_build_log.yml", Default::default())
    {
        error!("Could not initialize log4rs. {error}");
        return;
    }
    build_database::<PgDatabaseBuilder, _>(options).await
}
