pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

use sqlx::PgPool;

use crate::error::Result as WEResult;

use self::{
    executors::ExecutorsService, jobs::JobsService, task_queue::TaskQueueService,
    tasks::TasksService, workflow_runs::WorkflowRunsService, workflows::WorkflowsService,
};

pub fn create_executors_service(pool: &PgPool) -> WEResult<ExecutorsService> {
    Ok(ExecutorsService::new(pool))
}

pub fn create_workflow_runs_service(pool: &PgPool) -> WEResult<WorkflowRunsService> {
    Ok(WorkflowRunsService::new(pool))
}

pub fn create_task_queue_service(pool: &PgPool) -> WEResult<TaskQueueService> {
    Ok(TaskQueueService::new(pool))
}

pub fn create_jobs_service(pool: &PgPool) -> WEResult<JobsService> {
    Ok(JobsService::new(pool))
}

pub fn create_tasks_service(pool: &PgPool) -> WEResult<TasksService> {
    Ok(TasksService::new(pool))
}

pub fn create_workflows_service(pool: &PgPool) -> WEResult<WorkflowsService> {
    Ok(WorkflowsService::new(pool))
}
