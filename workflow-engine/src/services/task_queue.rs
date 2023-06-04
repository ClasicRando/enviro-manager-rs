use common::{
    database::Database,
    error::{EmError, EmResult},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    tasks::TaskId,
    workflow_runs::{WorkflowRunId, WorkflowRunsService},
};

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

/// Service for fetching and interacting with `task_queue` data. Wraps a [Pool] and provides
/// interaction methods for the API and [Executor][crate::executor::Executor] instances.
#[async_trait::async_trait]
pub trait TaskQueueService
where
    Self: Clone + Send + Sync + 'static,
{
    type Database: Database;
    type WorkflowRunService: WorkflowRunsService<Database = Self::Database>;

    /// Create a new [TaskQueueService] with the referenced pool as the data source
    fn create(
        pool: &<Self::Database as Database>::ConnectionPool,
        workflow_runs_service: &Self::WorkflowRunService,
    ) -> Self;
    /// Read a single task record from `task.task_queue` for the specified `request`data. Will
    /// return [Err] when the ids in the `request` do not match a record.
    async fn read_one(&self, request: &TaskQueueRequest) -> EmResult<TaskQueueRecord>;
    /// Append the task `rule` data to the specified `task_queue` record
    async fn append_task_rule(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: &i32,
        rule: TaskRule,
    ) -> EmResult<()>;
    /// Update the specified `task_queue` record with the new progress value
    async fn set_task_progress(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: &i32,
        progress: i16,
    ) -> EmResult<()>;
    /// Retry the specified `task_queue` record. Note, the record must be in the 'Failed' or
    /// 'Rule Broken' state to qualify for a retry.
    async fn retry_task(&self, request: &TaskQueueRequest) -> EmResult<()>;
    /// Complete the specified `task_queue` record to allow for continuing of a workflow run after
    /// a user interruption. Note, the record must be in the 'Paused' state for a successful
    /// complete.
    async fn complete_task(&self, request: &TaskQueueRequest) -> EmResult<()>;
    /// Acquire the next available task for a workflow run execution. Modifies the next available
    /// record to mark it as started. Will return [None] if there are no more available tasks to
    /// run.
    async fn next_task(&self, workflow_run_id: &WorkflowRunId)
        -> EmResult<Option<TaskQueueRecord>>;
    /// Run the specified task `record` to completion. See [TaskQueueService::remote_task_run] for
    /// more details. Remote task execution is run against the [Pool::close_event] so in the event
    /// of a pool close or database connection loss, the remote task execution is canceled.
    async fn run_task(&self, record: &TaskQueueRecord) -> EmResult<(bool, Option<String>)>;
    /// Mark the specified task `record` as failed with the error message included
    async fn fail_task_run(&self, record: &TaskQueueRecord, error: EmError) -> EmResult<()>;
    /// Complete the specified task `record` as complete (or paused if the `is_paused` flag is
    /// true). Includes an optional message if provided.
    async fn complete_task_run(
        &self,
        record: &TaskQueueRecord,
        is_paused: bool,
        message: Option<String>,
    ) -> EmResult<()>;
}
