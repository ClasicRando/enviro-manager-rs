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
    executors::ExecutorService,
    jobs::JobService,
    postgres::{
        executors::PgExecutorService, jobs::PgJobsService, tasks::PgTasksService,
        task_queue::PgTaskQueueService, workflow_runs::PgWorkflowRunsService,
        workflows::PgWorkflowsService,
    },
    task_queue::{TaskQueueRecord, TaskQueueService, TaskResponse},
    tasks::TaskService,
    workflow_runs::WorkflowRunsService,
    workflows::WorkflowsService,
};
