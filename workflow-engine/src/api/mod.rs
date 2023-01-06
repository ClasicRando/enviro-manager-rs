use rocket::{routes, Build, Config, Rocket};

use crate::{database::create_db_pool, Result as WEResult};

pub async fn build_api() -> WEResult<Rocket<Build>> {
    let pool = create_db_pool().await?;
    let config = Config {
        port: 8000,
        ..Default::default()
    };
    let build = rocket::build()
        .manage(pool)
        .configure(config)
        .mount("/api/v1/", routes![]);
    Ok(build)
}
