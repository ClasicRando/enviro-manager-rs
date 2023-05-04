pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

use common::error::EmResult;
use sqlx::PgPool;

use self::{
    executors::ExecutorsService, jobs::JobsService, task_queue::TaskQueueService,
    tasks::TasksService, workflow_runs::WorkflowRunsService, workflows::WorkflowsService,
};

pub fn create_executors_service(pool: &PgPool) -> EmResult<ExecutorsService> {
    Ok(ExecutorsService::new(pool))
}

pub fn create_workflow_runs_service(pool: &PgPool) -> EmResult<WorkflowRunsService> {
    Ok(WorkflowRunsService::new(pool))
}

pub fn create_task_queue_service(pool: &PgPool) -> EmResult<TaskQueueService> {
    Ok(TaskQueueService::new(pool))
}

pub fn create_jobs_service(pool: &PgPool) -> EmResult<JobsService> {
    Ok(JobsService::new(pool))
}

pub fn create_tasks_service(pool: &PgPool) -> EmResult<TasksService> {
    Ok(TasksService::new(pool))
}

pub fn create_workflows_service(pool: &PgPool) -> EmResult<WorkflowsService> {
    Ok(WorkflowsService::new(pool))
}
