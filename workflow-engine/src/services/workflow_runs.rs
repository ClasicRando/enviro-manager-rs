use std::str::FromStr;

use chrono::NaiveDateTime;
use common::error::{EmError, EmResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Database, Pool};

use crate::{
    database::listener::ChangeListener,
    executor::utilities::{WorkflowRunCancelMessage, WorkflowRunScheduledMessage},
    services::{
        executors::ExecutorId,
        task_queue::{TaskRule, TaskStatus},
        workflows::WorkflowId,
    },
};

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
    pub(crate) task_order: i32,
    pub(crate) task_id: i64,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) task_status: TaskStatus,
    pub(crate) parameters: Option<Value>,
    pub(crate) output: Option<String>,
    pub(crate) rules: Option<Vec<TaskRule>>,
    pub(crate) task_start: Option<NaiveDateTime>,
    pub(crate) task_end: Option<NaiveDateTime>,
    pub(crate) progress: Option<i16>,
}

/// Workflow run data as fetched from `workflow.v_workflow_runs`
#[derive(sqlx::FromRow, Serialize)]
pub struct WorkflowRun {
    pub(crate) workflow_run_id: WorkflowRunId,
    pub(crate) workflow_id: i64,
    pub(crate) status: WorkflowRunStatus,
    pub(crate) executor_id: Option<i64>,
    pub(crate) progress: i16,
    pub(crate) tasks: Vec<WorkflowRunTask>,
}

/// Workflow run data as fetched from the function `executor.all_executor_workflows`. Contains the
/// `workflow_run_id`, `status` of the workflow run and `is_valid` to denote if the workflow run is
/// valid when an [Executor][crate::executor::Executor] checks owned workflow runs. Valid workflow
/// runs are when there are only `task_queue` records for the workflow run that are 'Waiting' or
/// 'Complete'
#[derive(sqlx::FromRow)]
pub struct ExecutorWorkflowRun {
    pub(crate) workflow_run_id: WorkflowRunId,
    pub(crate) status: WorkflowRunStatus,
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

#[async_trait::async_trait]
pub trait WorkflowRunsService
where
    Self: Clone + Send + Sync + 'static
{
    type CancelListener: ChangeListener<WorkflowRunCancelMessage>;
    type Database: Database;
    type ScheduledListener: ChangeListener<WorkflowRunScheduledMessage>;

    /// Create a new [WorkflowRunsService] with the referenced pool as the data source
    fn create(pool: &Pool<Self::Database>) -> Self;
    /// Initialize a new workflow run for the specified `workflow_id`. Returns the new [WorkflowRun]
    /// instance.
    async fn initialize(&self, workflow_id: &WorkflowId) -> EmResult<WorkflowRun>;
    /// Read a single [WorkflowRun] record from `workflow.v_workflow_runs` for the specified
    /// `workflow_run_id`. Will return [Err] when the id does not match a record.
    async fn read_one(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun>;
    /// Read all [WorkflowRun] records found from `workflow.v_workflow_runs`
    async fn read_many(&self) -> EmResult<Vec<WorkflowRun>>;
    /// Update the status of the workflow run to 'Canceled' and send a notification to the
    /// [Executor][crate::executor::Executor] handling the workflow run to stop operations.
    async fn cancel(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun>;
    /// Schedule a workflow run to be picked up by an available
    /// [Executor][crate::executor::Executor]. Return a [WorkflowRun] with the new data from the
    /// scheduled record of `workflow_run_id`.
    async fn schedule(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun>;
    /// Schedule a workflow run to be picked up by the [Executor][crate::executor::Executor]
    /// specified by `executor_id`. Returns a [WorkflowRun] with the new data from the scheduled
    /// record of `workflow_run_id`.
    async fn schedule_with_executor(
        &self,
        workflow_run_id: &WorkflowRunId,
        executor_id: &ExecutorId,
    ) -> EmResult<WorkflowRun>;
    /// Restart a workflow run to a 'Waiting' state. Copies current state of the `task_queue` before
    /// updating restarting all tasks and the workflow run itself. Returns a [WorkflowRun] with the
    /// new state of the workflow run for the specified `workflow_run_id`.
    async fn restart(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun>;
    /// Update the progress of a workflow run. The progress is not provided but rather calculated
    /// by the progress of it's tasks.
    async fn update_progress(&self, workflow_run_id: &WorkflowRunId) -> EmResult<()>;
    /// Complete a workflow run by collecting stats about the run's tasks and updating the status
    /// of the workflow run accordingly.
    async fn complete(&self, workflow_run_id: &WorkflowRunId) -> EmResult<()>;
    /// Fetch all workflow runs attached to an executor specified by `executor_id`.
    async fn all_executor_workflows(
        &self,
        executor_id: &ExecutorId,
    ) -> EmResult<Vec<ExecutorWorkflowRun>>;
    /// Start the move of a workflow run to another executor (or back to the 'Scheduled' workflow
    /// run pool if no executors are available). Updates the next task up for execution to the
    /// 'Paused' status. Returns the new state of the [WorkflowRun] specified by `workflow_run_id`.
    async fn start_move(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun>;
    /// Complete the move of a workflow run to another executor (or back to the 'Scheduled' workflow
    /// run pool if no executors are available). Updates the next task with a 'Paused' status to the
    /// 'Waiting' status and schedules the workflow run for execution. Returns the new state of the
    /// [WorkflowRun] specified by `workflow_run_id`.
    async fn complete_move(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun>;
    /// Get a new workflow run scheduled listener for the specified `executor_id`. The
    /// [ChangeListener] checks a channel named `wr_scheduled_{executor_id}`
    async fn scheduled_listener(
        &self,
        executor_id: &ExecutorId,
    ) -> EmResult<Self::ScheduledListener>;
    /// Get a new workflow run canceled listener for the specified `executor_id`. The
    /// [ChangeListener] checks a channel named `wr_canceled_{executor_id}`
    async fn cancel_listener(&self, executor_id: &ExecutorId) -> EmResult<Self::CancelListener>;
}
