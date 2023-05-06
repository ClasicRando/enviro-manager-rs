use std::str::FromStr;

use common::error::EmError;
use log::warn;
use tokio::task::JoinHandle;

use crate::services::workflow_runs::WorkflowRunId;

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
impl FromStr for ExecutorStatusUpdate {
    type Err = EmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "cancel" => Self::Cancel,
            "shutdown" => Self::Shutdown,
            _ => Self::NoOp,
        })
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

pub struct WorkflowRunCancelMessage(pub Option<WorkflowRunId>);

impl FromStr for WorkflowRunCancelMessage {
    type Err = EmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(workflow_run_id) => Ok(Self(Some(workflow_run_id))),
            Err(error) => {
                warn!("Cannot parse workflow_run_id from `{}`. {}", s, error);
                Ok(Self(None))
            }
        }
    }
}

pub struct WorkflowRunScheduledMessage;

impl FromStr for WorkflowRunScheduledMessage {
    type Err = EmError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}
