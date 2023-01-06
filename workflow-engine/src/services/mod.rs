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
    workflow_runs::WorkflowRunsService,
};

static EXECUTORS_SERVICE: OnceCell<ExecutorsService> = OnceCell::new();
static WORKFLOW_RUNS_SERVICE: OnceCell<WorkflowRunsService> = OnceCell::new();
static TASK_QUEUE_SERVICE: OnceCell<TaskQueueService> = OnceCell::new();
static JOBS_SERVICE: OnceCell<JobsService> = OnceCell::new();

pub async fn executors_service() -> WEResult<&'static ExecutorsService> {
    EXECUTORS_SERVICE
        .get_or_try_init(async {
            let pool = we_db_pool().await?;
            Ok(ExecutorsService::new(pool))
        })
        .await
}

pub async fn workflow_runs_service() -> WEResult<&'static WorkflowRunsService> {
    WORKFLOW_RUNS_SERVICE
        .get_or_try_init(async {
            let pool = we_db_pool().await?;
            Ok(WorkflowRunsService::new(pool))
        })
        .await
}

pub async fn task_queue_service() -> WEResult<&'static TaskQueueService> {
    TASK_QUEUE_SERVICE
        .get_or_try_init(async {
            let pool = we_db_pool().await?;
            Ok(TaskQueueService::new(pool))
        })
        .await
}

pub async fn jobs_service() -> WEResult<&'static JobsService> {
    JOBS_SERVICE
        .get_or_try_init(async {
            let pool = we_db_pool().await?;
            Ok(JobsService::new(pool))
        })
        .await
}
