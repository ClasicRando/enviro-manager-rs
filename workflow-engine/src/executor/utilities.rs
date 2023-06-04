//! Utilities module for components of an [Executor][crate::executor::Executor]

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
    pub const fn is_cancelled(&self) -> bool {
        match self {
            Self::Cancel => true,
            Self::Shutdown | Self::NoOp => false,
        }
    }
}

/// Container for a notification message indicating that a workflow run is to be cancelled. If the
/// inner content is [None] then the message was not valid and should be ignored.
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

/// Unit struct to represent that a workflow run was scheduled and the
/// [`Executor`][crate::executor::Executor] should restart from a listen state to active. Message
/// contents are ignored and [Ok] is always returned
pub struct WorkflowRunScheduledMessage;

impl FromStr for WorkflowRunScheduledMessage {
    type Err = EmError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}
