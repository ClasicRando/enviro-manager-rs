use tokio::task::JoinHandle;

use crate::error::Error;

pub type WorkflowRunWorkerResult = JoinHandle<(i64, Option<Error>)>;

pub enum ExecutorNotificationSignal {
    Cancel,
    Shutdown,
    Cleanup,
    NoOp,
    Error(sqlx::Error),
}
