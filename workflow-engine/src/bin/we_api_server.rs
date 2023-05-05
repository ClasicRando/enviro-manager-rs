use actix_web::{
    web::{get, Data},
    App, HttpServer,
};
use common::error::EmResult;
use workflow_engine::{
    api, create_executors_service, create_jobs_service, create_task_queue_service,
    create_tasks_service, create_workflow_runs_service, create_workflows_service,
    database::create_db_pool, PgExecutorsService, PgJobsService, PgTaskQueueService,
    PgTasksService, PgWorkflowRunsService, PgWorkflowsService,
};

#[tokio::main]
async fn main() -> EmResult<()> {
    log4rs::init_file("workflow-engine/api_server_log.yml", Default::default()).unwrap();
    let pool = create_db_pool().await?;
    let executors_service: Data<PgExecutorsService> = Data::new(create_executors_service(&pool)?);
    let jobs_service: Data<PgJobsService> = Data::new(create_jobs_service(&pool)?);
    let task_queue_service: Data<PgTaskQueueService> = Data::new(create_task_queue_service(&pool)?);
    let tasks_service: Data<PgTasksService> = Data::new(create_tasks_service(&pool)?);
    let workflow_runs_service: Data<PgWorkflowRunsService> =
        Data::new(create_workflow_runs_service(&pool)?);
    let workflows_service: Data<PgWorkflowsService> = Data::new(create_workflows_service(&pool)?);
    let server = HttpServer::new(move || {
        App::new().service(
            actix_web::web::scope("/api/v1")
                .app_data(executors_service.clone())
                .app_data(jobs_service.clone())
                .app_data(task_queue_service.clone())
                .app_data(tasks_service.clone())
                .app_data(workflow_runs_service.clone())
                .app_data(workflows_service.clone())
                .route(
                    "/executors",
                    get().to(api::executors::active_executors::<PgExecutorsService>),
                )
                .route(
                    "/executors/shutdown/{executor_id}",
                    get().to(api::executors::shutdown_executor::<PgExecutorsService>),
                )
                .route(
                    "/executors/cancel/{executor_id}",
                    get().to(api::executors::cancel_executor::<PgExecutorsService>),
                )
        )
    })
    .bind(("127.0.0.1", 8080))?;
    server.run().await?;
    Ok(())
}
