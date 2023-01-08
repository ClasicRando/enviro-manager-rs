use workflow_engine::build_api;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = build_api().await.unwrap().launch().await?;
    Ok(())
}
