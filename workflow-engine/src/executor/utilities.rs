use common::error::EmError;
use tokio::task::JoinHandle;

use crate::{database::listener::FromPayload, services::workflow_runs::WorkflowRunId};

/// Type alias for a workflow run worker result. Represents a tokio task [JoinHandle] returning a
/// tuple of [WorkflowRunId] and an optional error if the workflow run failed.
pub type WorkflowRunWorkerResult = JoinHandle<(WorkflowRunId, Option<EmError>)>;

/// Executor status notification payload values
#[derive(PartialEq, Debug)]
pub enum ExecutorStatusUpdate {
    Cancel,
    Shutdown,
    NoOp,
}

impl FromPayload for ExecutorStatusUpdate {
    fn from_payload(payload: &str) -> Self {
        match payload {
            "cancel" => Self::Cancel,
            "shutdown" => Self::Shutdown,
            _ => Self::NoOp,
        }
    }
}

impl From<&str> for ExecutorStatusUpdate {
    fn from(value: &str) -> Self {
        match value {
            "cancel" => Self::Cancel,
            "shutdown" => Self::Shutdown,
            _ => Self::NoOp,
        }
    }
}

impl ExecutorStatusUpdate {
    /// True if the value represents a cancellation notification
    pub fn is_cancelled(&self) -> bool {
        match self {
            ExecutorStatusUpdate::Cancel => true,
            ExecutorStatusUpdate::Shutdown | ExecutorStatusUpdate::NoOp => false,
        }
    }
}
