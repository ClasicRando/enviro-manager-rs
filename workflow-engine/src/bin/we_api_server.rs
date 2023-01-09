use workflow_engine::build_api;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    log4rs::init_file("workflow-engine/api_server_log.yml", Default::default()).unwrap();

    let _ = build_api().await.unwrap().launch().await?;
    Ok(())
}
