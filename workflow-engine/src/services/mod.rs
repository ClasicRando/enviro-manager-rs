pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

use once_cell::sync::OnceCell;
use sqlx::PgPool;

use crate::{database::we_db_pool, error::Result as WEResult};

use self::{
    executors::ExecutorsService, jobs::JobsService, task_queue::TaskQueueService,
    tasks::TasksService, workflow_runs::WorkflowRunsService, workflows::WorkflowsService,
};

static EXECUTORS_SERVICE: OnceCell<ExecutorsService> = OnceCell::new();
static WORKFLOW_RUNS_SERVICE: OnceCell<WorkflowRunsService> = OnceCell::new();
static TASK_QUEUE_SERVICE: OnceCell<TaskQueueService> = OnceCell::new();
static JOBS_SERVICE: OnceCell<JobsService> = OnceCell::new();
static TASKS_SERVICE: OnceCell<TasksService> = OnceCell::new();
static WORKFLOWS_SERVICE: OnceCell<WorkflowsService> = OnceCell::new();

pub fn create_executors_service(pool: &PgPool) -> WEResult<ExecutorsService> {
    Ok(ExecutorsService::new(pool))
}

pub async fn executors_service() -> WEResult<&'static ExecutorsService> {
    let pool = we_db_pool().await?;
    EXECUTORS_SERVICE.get_or_try_init(move || create_executors_service(&pool))
}

pub fn create_workflow_runs_service(pool: &PgPool) -> WEResult<WorkflowRunsService> {
    Ok(WorkflowRunsService::new(pool))
}

pub async fn workflow_runs_service() -> WEResult<&'static WorkflowRunsService> {
    let pool = we_db_pool().await?;
    WORKFLOW_RUNS_SERVICE.get_or_try_init(move || create_workflow_runs_service(&pool))
}

pub fn create_task_queue_service(pool: &PgPool) -> WEResult<TaskQueueService> {
    Ok(TaskQueueService::new(pool))
}

pub async fn task_queue_service() -> WEResult<&'static TaskQueueService> {
    let pool = we_db_pool().await?;
    TASK_QUEUE_SERVICE.get_or_try_init(move || create_task_queue_service(&pool))
}

pub fn create_jobs_service(pool: &PgPool) -> WEResult<JobsService> {
    Ok(JobsService::new(pool))
}

pub async fn jobs_service() -> WEResult<&'static JobsService> {
    let pool = we_db_pool().await?;
    JOBS_SERVICE.get_or_try_init(move || create_jobs_service(&pool))
}

pub fn create_tasks_service(pool: &PgPool) -> WEResult<TasksService> {
    Ok(TasksService::new(pool))
}

pub async fn tasks_service() -> WEResult<&'static TasksService> {
    let pool = we_db_pool().await?;
    TASKS_SERVICE.get_or_try_init(move || create_tasks_service(&pool))
}

pub fn create_workflows_service(pool: &PgPool) -> WEResult<WorkflowsService> {
    Ok(WorkflowsService::new(pool))
}

pub async fn workflows_service() -> WEResult<&'static WorkflowsService> {
    let pool = we_db_pool().await?;
    WORKFLOWS_SERVICE.get_or_try_init(move || create_workflows_service(&pool))
}
