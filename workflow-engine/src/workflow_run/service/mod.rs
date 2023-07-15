pub mod postgres;

use common::{
    database::{listener::ChangeListener, Database},
    error::{EmError, EmResult},
};

use super::data::{
    ExecutorWorkflowRun, TaskQueueRecord, TaskQueueRequest, TaskRule, WorkflowRun, WorkflowRunId,
};
use crate::{
    executor::{
        data::ExecutorId,
        utilities::{WorkflowRunCancelMessage, WorkflowRunScheduledMessage},
    },
    workflow::{data::WorkflowId, service::WorkflowsService},
};

#[async_trait::async_trait]
pub trait WorkflowRunsService
where
    Self: Clone + Send + Sync + 'static,
{
    type CancelListener: ChangeListener<Message = WorkflowRunCancelMessage>;
    type Database: Database;
    type ScheduledListener: ChangeListener<Message = WorkflowRunScheduledMessage>;
    type WorkflowService: WorkflowsService;

    /// Initialize a new workflow run for the specified `workflow_id`. Returns the new [WorkflowRun]
    /// instance.
    async fn initialize(&self, workflow_id: &WorkflowId) -> EmResult<WorkflowRun>;
    /// Read a single [WorkflowRun] record from `workflow.v_workflow_runs` for the specified
    /// `workflow_run_id`. Will return [Err] when the id does not match a record.
    async fn read_one(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun>;
    /// Read all [WorkflowRun] records found from `workflow.v_workflow_runs`
    async fn read_active(&self) -> EmResult<Vec<WorkflowRun>>;
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

/// Service for fetching and interacting with `task_queue` data. Wraps a [Pool] and provides
/// interaction methods for the API and [Executor][crate::executor::Executor] instances.
#[async_trait::async_trait]
pub trait TaskQueueService
where
    Self: Clone + Send + Sync + 'static,
{
    type Database: Database;
    type WorkflowRunService: WorkflowRunsService<Database = Self::Database>;

    /// Read a single task record from `task.task_queue` for the specified `request`data. Will
    /// return [Err] when the ids in the `request` do not match a record.
    async fn read_one(&self, request: &TaskQueueRequest) -> EmResult<TaskQueueRecord>;
    /// Append the task `rule` data to the specified `task_queue` record
    async fn append_task_rule(&self, request: &TaskQueueRequest, rule: &TaskRule) -> EmResult<()>;
    /// Update the specified `task_queue` record with the new progress value
    async fn set_task_progress(&self, request: &TaskQueueRequest, progress: i16) -> EmResult<()>;
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
