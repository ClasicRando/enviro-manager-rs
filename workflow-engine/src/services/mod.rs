pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;

use common::error::EmResult;
use sqlx::{Database, PgPool, Pool};

use self::{
    jobs::JobsService, task_queue::TaskQueueService, tasks::TasksService,
    workflow_runs::WorkflowRunsService, workflows::WorkflowsService,
};
use crate::ExecutorsService;

pub fn create_executors_service<E: ExecutorsService<Database = D>, D: Database>(
    pool: &Pool<D>,
) -> EmResult<E> {
    Ok(E::new(pool))
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
