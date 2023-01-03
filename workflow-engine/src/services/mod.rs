pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

use async_once_cell::OnceCell;

use crate::{database::we_db_pool, error::Result as WEResult};

use self::{
    executors::ExecutorsService, task_queue::TaskQueueService, workflow_runs::WorkflowRunsService,
};

static EXECUTORS_SERVICE: OnceCell<ExecutorsService> = OnceCell::new();
static WORKFLOW_RUNS_SERVICE: OnceCell<WorkflowRunsService> = OnceCell::new();
static TASK_QUEUE_SERVICE: OnceCell<task_queue::TaskQueueService> = OnceCell::new();

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
