pub mod executors;
pub mod jobs;
pub mod task_queue;
pub mod tasks;
pub mod workflow_runs;
pub mod workflows;
pub mod postgres;

use common::error::EmResult;
use sqlx::{Database, Pool};

use self::{
    executors::ExecutorService, jobs::JobsService, task_queue::TaskQueueService,
    tasks::TasksService, workflow_runs::WorkflowRunsService, workflows::WorkflowsService,
};

pub fn create_workflow_runs_service<R: WorkflowRunsService<Database = D>, D: Database>(
    pool: &Pool<D>,
) -> EmResult<R> {
    Ok(R::new(pool))
}

pub fn create_task_queue_service<Q: TaskQueueService<Database = D>, D: Database>(
    pool: &Pool<D>,
) -> EmResult<Q> {
    Ok(Q::new(pool))
}

pub fn create_jobs_service<J: JobsService<Database = D>, D: Database>(
    pool: &Pool<D>,
) -> EmResult<J> {
    Ok(J::new(pool))
}

pub fn create_tasks_service<T: TasksService<Database = D>, D: Database>(
    pool: &Pool<D>,
) -> EmResult<T> {
    Ok(T::new(pool))
}

pub fn create_workflows_service<W: WorkflowsService<Database = D>, D: Database>(
    pool: &Pool<D>,
) -> EmResult<W> {
    Ok(W::new(pool))
}
