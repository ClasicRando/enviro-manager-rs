use tokio::task::JoinHandle;

use crate::{error::Error, services::workflow_runs::WorkflowRunId};

pub type WorkflowRunWorkerResult = JoinHandle<(WorkflowRunId, Option<Error>)>;

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
    pub fn is_cancelled(&self) -> bool {
        match self {
            ExecutorNotificationSignal::Cancel => true,
            ExecutorNotificationSignal::Shutdown
            | ExecutorNotificationSignal::NoOp
            | ExecutorNotificationSignal::Cleanup => false,
        }
    }
}
