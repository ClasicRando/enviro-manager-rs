use tokio::task::JoinHandle;

use crate::{error::Error, services::workflow_runs::WorkflowRunId};

pub type WorkflowRunWorkerResult = JoinHandle<(WorkflowRunId, Option<Error>)>;

pub enum ExecutorNotificationSignal {
    Cancel,
    Shutdown,
    Cleanup,
    NoOp,
    Error(sqlx::Error),
}
