use std::str::FromStr;

use chrono::NaiveDateTime;
use common::error::EmError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::tasks::TaskId;

/// Status of a workflow run as found in the database as a simple Postgresql enum type
#[derive(sqlx::Type, PartialEq, Eq, Serialize)]
#[sqlx(type_name = "workflow_run_status")]
pub enum WorkflowRunStatus {
    Waiting,
    Scheduled,
    Running,
    Paused,
    Failed,
    Complete,
    Canceled,
}

/// Task information for entries under a [WorkflowRun]
#[derive(Serialize)]
pub struct WorkflowRunTask {
    /// Order of a task within a workflow run
    pub(crate) task_order: i32,
    /// ID of the task to be executed
    pub(crate) task_id: TaskId,
    /// Name of the task
    pub(crate) name: String,
    /// Short description of the task
    pub(crate) description: String,
    /// Status of the task
    pub(crate) task_status: TaskStatus,
    /// Optional parameters passed to the task executor to allow for custom behaviour
    pub(crate) parameters: Option<Value>,
    /// Optional output message for the task
    pub(crate) output: Option<String>,
    /// Optional list of task rules for the workflow run task
    pub(crate) rules: Option<Vec<TaskRule>>,
    /// Start of the task execution
    pub(crate) task_start: Option<NaiveDateTime>,
    /// End of the task execution
    pub(crate) task_end: Option<NaiveDateTime>,
    /// Optional progress value passed back from the task executor
    pub(crate) progress: Option<i16>,
}

/// Workflow run data as fetched from `workflow.v_workflow_runs`
#[derive(sqlx::FromRow, Serialize)]
pub struct WorkflowRun {
    /// ID of the workflow run
    pub(crate) workflow_run_id: WorkflowRunId,
    /// ID of the workflow that is executed for this workflow run
    pub(crate) workflow_id: i64,
    /// Status of the workflow run
    pub(crate) status: WorkflowRunStatus,
    /// Optional ID of the executor that owns this workflow run, [None] if not currently running
    pub(crate) executor_id: Option<i64>,
    /// Optional Progress of the workflow run
    pub(crate) progress: Option<i16>,
    /// Tasks that are part of this workflow run
    pub(crate) tasks: Vec<WorkflowRunTask>,
}

/// Workflow run data as fetched from the function `executor.all_executor_workflows`. Contains the
/// `workflow_run_id`, `status` of the workflow run and `is_valid` to denote if the workflow run is
/// valid when an [Executor][crate::executor::Executor] checks owned workflow runs.
#[derive(sqlx::FromRow)]
pub struct ExecutorWorkflowRun {
    /// ID of the workflow run
    pub(crate) workflow_run_id: WorkflowRunId,
    /// Status of the workflow run
    pub(crate) status: WorkflowRunStatus,
    /// Flag indicating if the workflow run is valid. Valid workflow runs are when there are only
    /// `task_queue` records for the workflow run that are 'Waiting' or 'Complete'
    pub(crate) is_valid: bool,
}

/// Wrapper for a `workflow_run_id` value. Made to ensure data passed as the id of a workflow run is
/// correct and not just any i64 value.
#[derive(sqlx::Type, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Debug)]
#[sqlx(transparent)]
pub struct WorkflowRunId(i64);

impl FromStr for WorkflowRunId {
    type Err = EmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<i64>()?.into())
    }
}

impl From<i64> for WorkflowRunId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for WorkflowRunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of a task as found in the database as a simple Postgresql enum type
#[derive(sqlx::Type, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[sqlx(type_name = "task_status")]
pub enum TaskStatus {
    Waiting,
    Running,
    Complete,
    Failed,
    #[sqlx(rename = "Rule Broken")]
    #[serde(rename = "Rule Broken")]
    RuleBroken,
    Paused,
    Canceled,
}

/// Check performed during a task run to validate the current state of a task or the system that the
/// task is operating on. Rules must always have a non-empty and unique `name` per task, as well as
/// a `failed` status and optional `message` to provide details of what the rule checked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRule {
    /// Descriptive name of the task rule
    pub(crate) name: String,
    /// Flag indicating if the task rule failed during the check
    pub(crate) failed: bool,
    /// Optional message included in the task rule completion
    pub(crate) message: Option<String>,
}

/// Represents a row from the `task.task_queue` table
#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct TaskQueueRecord {
    /// ID of the Workflow run that owns this task queue record
    pub(crate) workflow_run_id: WorkflowRunId,
    /// Order within the workflow run
    pub(crate) task_order: i32,
    /// ID of the task that is executed
    pub(crate) task_id: TaskId,
    /// Status of the task
    pub(crate) status: TaskStatus,
    /// Parameters passed to the task to modify behaviour
    pub(crate) parameters: Option<Value>,
    /// Url to be called as per the task execution
    pub(crate) url: String,
}

/// Container for the data required to fetch/update a single `task.task_queue` record
#[derive(Deserialize)]
pub struct TaskQueueRequest {
    /// ID of the  workflow run to be accessed
    pub(crate) workflow_run_id: WorkflowRunId,
    /// Order within the workflow run
    pub(crate) task_order: i32,
}

/// Container for the various task run responses a task execution service can stream back to an
/// [Executor][crate::executor::Executor]. The responses are a [TaskResponse::Progress] update
/// (0-100%), a [TaskResponse::Rule] check that has completed or the terminal [TaskResponse::Done]
/// message that contains a success flag and an optional message.
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Progress(i16),
    Rule(TaskRule),
    Done {
        success: bool,
        message: Option<String>,
    },
}
