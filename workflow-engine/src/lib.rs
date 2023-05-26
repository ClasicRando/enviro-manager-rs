#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

pub mod api;
pub mod database;
mod error;
mod executor;
mod job_worker;
mod services;

pub use error::{Error, Result};
pub use executor::Executor;
pub use job_worker::JobWorker;
pub use services::{
    create_task_queue_service, create_tasks_service, create_workflow_runs_service,
    create_workflows_service,
    executors::ExecutorService,
    jobs::JobsService,
    postgres::{executors::PgExecutorService, jobs::PgJobsService},
    task_queue::{PgTaskQueueService, TaskQueueRecord, TaskQueueService, TaskResponse},
    tasks::{PgTasksService, TasksService},
    workflow_runs::{PgWorkflowRunsService, WorkflowRunsService},
    workflows::{PgWorkflowsService, WorkflowsService},
};
