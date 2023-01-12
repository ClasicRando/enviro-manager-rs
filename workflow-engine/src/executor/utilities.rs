use tokio::task::JoinHandle;

use crate::{error::Error, services::workflow_runs::WorkflowRunId};

/// Type alias for a workflow run worker result. Represents a tokio task [JoinHandle] returning a
/// tuple of [WorkflowRunId] and an optional error if the workflow run failed.
pub type WorkflowRunWorkerResult = JoinHandle<(WorkflowRunId, Option<Error>)>;

/// Executor status notification payload values
pub enum ExecutorNotificationSignal {
    Cancel,
    Shutdown,
    Cleanup,
    NoOp,
}

impl From<&str> for ExecutorNotificationSignal {
    fn from(value: &str) -> Self {
        match value {
            "cancel" => Self::Cancel,
            "shutdown" => Self::Shutdown,
            "cleanup" => Self::Cleanup,
            _ => Self::NoOp,
        }
    }
}

impl ExecutorNotificationSignal {
    /// True if the value represents a cancellation notification
    pub fn is_cancelled(&self) -> bool {
        match self {
            ExecutorNotificationSignal::Cancel => true,
            ExecutorNotificationSignal::Shutdown
            | ExecutorNotificationSignal::NoOp
            | ExecutorNotificationSignal::Cleanup => false,
        }
    }
}
