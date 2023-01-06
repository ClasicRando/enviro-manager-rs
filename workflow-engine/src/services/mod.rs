pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

use async_once_cell::OnceCell;

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

pub async fn create_executors_service() -> WEResult<ExecutorsService> {
    let pool = we_db_pool().await?;
    Ok(ExecutorsService::new(pool))
}

pub async fn executors_service() -> WEResult<&'static ExecutorsService> {
    EXECUTORS_SERVICE
        .get_or_try_init(create_executors_service())
        .await
}

pub async fn create_workflow_runs_service() -> WEResult<WorkflowRunsService> {
    let pool = we_db_pool().await?;
    Ok(WorkflowRunsService::new(pool))
}

pub async fn workflow_runs_service() -> WEResult<&'static WorkflowRunsService> {
    WORKFLOW_RUNS_SERVICE
        .get_or_try_init(create_workflow_runs_service())
        .await
}

pub async fn create_task_queue_service() -> WEResult<TaskQueueService> {
    let pool = we_db_pool().await?;
    Ok(TaskQueueService::new(pool))
}

pub async fn task_queue_service() -> WEResult<&'static TaskQueueService> {
    TASK_QUEUE_SERVICE
        .get_or_try_init(create_task_queue_service())
        .await
}

pub async fn create_jobs_service() -> WEResult<JobsService> {
    let pool = we_db_pool().await?;
    Ok(JobsService::new(pool))
}

pub async fn jobs_service() -> WEResult<&'static JobsService> {
    JOBS_SERVICE.get_or_try_init(create_jobs_service()).await
}

pub async fn create_tasks_service() -> WEResult<TasksService> {
    let pool = we_db_pool().await?;
    Ok(TasksService::new(pool))
}


pub async fn tasks_service() -> WEResult<&'static TasksService> {
    TASKS_SERVICE
        .get_or_try_init(create_tasks_service())
        .await
}

pub async fn create_workflows_service() -> WEResult<WorkflowsService> {
    let pool = we_db_pool().await?;
    Ok(WorkflowsService::new(pool))
}


pub async fn workflows_service() -> WEResult<&'static WorkflowsService> {
    WORKFLOWS_SERVICE
        .get_or_try_init(create_workflows_service())
        .await
}
