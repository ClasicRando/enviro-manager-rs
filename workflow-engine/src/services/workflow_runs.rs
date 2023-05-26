use std::str::FromStr;

use chrono::NaiveDateTime;
use common::error::{EmError, EmResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgListener, PgTypeInfo, PgValueRef,
    },
    Database, PgPool, Pool, Postgres, Type,
};

use super::{executors::ExecutorId, task_queue::TaskRule, tasks::TaskStatus};
use crate::{
    database::listener::{ChangeListener, PgChangeListener},
    executor::utilities::{WorkflowRunCancelMessage, WorkflowRunScheduledMessage},
    services::workflows::WorkflowId,
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
    task_order: i32,
    task_id: i64,
    name: String,
    description: String,
    task_status: TaskStatus,
    parameters: Option<Value>,
    output: Option<String>,
    rules: Option<Vec<TaskRule>>,
    task_start: Option<NaiveDateTime>,
    task_end: Option<NaiveDateTime>,
    progress: Option<i16>,
}

impl Encode<'_, Postgres> for WorkflowRunTask {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);
        encoder.encode(self.task_order);
        encoder.encode(self.task_id);
        encoder.encode(&self.name);
        encoder.encode(&self.description);
        encoder.encode(&self.task_status);
        encoder.encode(&self.parameters);
        encoder.encode(&self.output);
        encoder.encode(&self.rules);
        encoder.encode(self.task_start);
        encoder.encode(self.task_end);
        encoder.encode(self.progress);
        encoder.finish();
        IsNull::No
    }

    fn size_hint(&self) -> usize {
        9usize * (4 + 4)
            + <i32 as Encode<Postgres>>::size_hint(&self.task_order)
            + <i64 as Encode<Postgres>>::size_hint(&self.task_id)
            + <String as Encode<Postgres>>::size_hint(&self.name)
            + <String as Encode<Postgres>>::size_hint(&self.description)
            + <TaskStatus as Encode<Postgres>>::size_hint(&self.task_status)
            + <Option<Value> as Encode<Postgres>>::size_hint(&self.parameters)
            + <Option<String> as Encode<Postgres>>::size_hint(&self.output)
            + <Option<Vec<TaskRule>> as Encode<Postgres>>::size_hint(&self.rules)
            + <Option<NaiveDateTime> as Encode<Postgres>>::size_hint(&self.task_start)
            + <Option<NaiveDateTime> as Encode<Postgres>>::size_hint(&self.task_end)
            + <Option<i16> as Encode<Postgres>>::size_hint(&self.progress)
    }
}

impl<'r> Decode<'r, Postgres> for WorkflowRunTask {
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let mut decoder = PgRecordDecoder::new(value)?;
        let task_order = decoder.try_decode::<i32>()?;
        let task_id = decoder.try_decode::<i64>()?;
        let name = decoder.try_decode::<String>()?;
        let description = decoder.try_decode::<String>()?;
        let task_status = decoder.try_decode::<TaskStatus>()?;
        let parameters = decoder.try_decode::<Option<Value>>()?;
        let output = decoder.try_decode::<Option<String>>()?;
        let rules = decoder.try_decode::<Option<Vec<TaskRule>>>()?;
        let task_start = decoder.try_decode::<Option<NaiveDateTime>>()?;
        let task_end = decoder.try_decode::<Option<NaiveDateTime>>()?;
        let progress = decoder.try_decode::<Option<i16>>()?;
        Ok(WorkflowRunTask {
            task_order,
            task_id,
            name,
            description,
            task_status,
            parameters,
            output,
            rules,
            task_start,
            task_end,
            progress,
        })
    }
}

impl Type<Postgres> for WorkflowRunTask {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("workflow_run_task")
    }
}

impl PgHasArrayType for WorkflowRunTask {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_run_task")
    }
}

/// Workflow run data as fetched from `workflow.v_workflow_runs`
#[derive(sqlx::FromRow, Serialize)]
pub struct WorkflowRun {
    workflow_run_id: i64,
    workflow_id: i64,
    status: WorkflowRunStatus,
    executor_id: Option<i64>,
    progress: i16,
    tasks: Vec<WorkflowRunTask>,
}

/// Workflow run data as fetched from the function `executor.all_executor_workflows`. Contains the
/// `workflow_run_id`, `status` of the workflow run and `is_valid` to denote if the workflow run is
/// valid when an [Executor][crate::executor::Executor] checks owned workflow runs. Valid workflow
/// runs are when there are only `task_queue` records for the workflow run that are 'Waiting' or
/// 'Complete'
#[derive(sqlx::FromRow)]
pub struct ExecutorWorkflowRun {
    pub workflow_run_id: WorkflowRunId,
    pub status: WorkflowRunStatus,
    pub is_valid: bool,
}

/// Wrapper for a `workflow_run_id` value. Made to ensure data passed as the id of a workflow run is
/// correct and not just any i64 value.
#[derive(sqlx::Type, Eq, PartialEq, Hash, Clone, Deserialize)]
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
pub trait WorkflowRunsService: Clone + Send + Sync + 'static {
    type CancelListener: ChangeListener<WorkflowRunCancelMessage>;
    type Database: Database;
    type ScheduledListener: ChangeListener<WorkflowRunScheduledMessage>;

    /// Create a new [WorkflowRunsService] with the referenced pool as the data source
    fn new(pool: &Pool<Self::Database>) -> Self;
    /// Initialize a new workflow run for the specified `workflow_id`. Returns the new [WorkflowRun]
    /// instance.
    async fn initialize(&self, workflow_id: &WorkflowId) -> EmResult<WorkflowRun>;
    /// Read a single [WorkflowRun] record from `workflow.v_workflow_runs` for the specified
    /// `workflow_run_id`. Will return [None] when the id does not match a record.
    async fn read_one(&self, workflow_run_id: &WorkflowRunId) -> EmResult<Option<WorkflowRun>>;
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
    /// Get a new workflow run scheduled listener for the specified `executor_id`. The [PgListener]
    /// checks a channel named `wr_scheduled_{executor_id}`
    async fn scheduled_listener(
        &self,
        executor_id: &ExecutorId,
    ) -> EmResult<Self::ScheduledListener>;
    /// Get a new workflow run canceled listener for the specified `executor_id`. The [PgListener]
    /// checks a channel named `wr_canceled_{executor_id}`
    async fn cancel_listener(&self, executor_id: &ExecutorId) -> EmResult<Self::CancelListener>;
}

/// Service for fetching and interacting with workflow run data. Wraps a [PgPool] and provides
/// interaction methods for the API and [Executor][crate::executor::Executor] instances.
#[derive(Clone)]
pub struct PgWorkflowRunsService {
    pool: PgPool,
}

#[async_trait::async_trait]
impl WorkflowRunsService for PgWorkflowRunsService {
    type CancelListener = PgChangeListener<WorkflowRunCancelMessage>;
    type Database = Postgres;
    type ScheduledListener = PgChangeListener<WorkflowRunScheduledMessage>;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn initialize(&self, workflow_id: &WorkflowId) -> EmResult<WorkflowRun> {
        let workflow_run_id = sqlx::query_scalar("select workflow.initialize_workflow_run($1)")
            .bind(workflow_id)
            .fetch_one(&self.pool)
            .await?;
        match self.read_one(&workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn read_one(&self, workflow_run_id: &WorkflowRunId) -> EmResult<Option<WorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select
                wr.workflow_run_id, wr.workflow_id, wr.status, wr.executor_id, wr.progress,
                wr.tasks
            from workflow.v_workflow_runs wr
            where workflow_run_id = $1"#,
        )
        .bind(workflow_run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    async fn read_many(&self) -> EmResult<Vec<WorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select
                wr.workflow_run_id, wr.workflow_id, wr.status, wr.executor_id, wr.progress,
                wr.tasks
            from workflow.v_workflow_runs wr"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn cancel(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.cancel_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        match self.read_one(workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn schedule(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.schedule_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        match self.read_one(workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn schedule_with_executor(
        &self,
        workflow_run_id: &WorkflowRunId,
        executor_id: &ExecutorId,
    ) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.schedule_workflow_run($1,$2)")
            .bind(workflow_run_id)
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        match self.read_one(workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn restart(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.restart_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        match self.read_one(workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn complete(&self, workflow_run_id: &WorkflowRunId) -> EmResult<()> {
        sqlx::query("call workflow.complete_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn all_executor_workflows(
        &self,
        executor_id: &ExecutorId,
    ) -> EmResult<Vec<ExecutorWorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select workflow_run_id, status, is_valid
            from executor.all_executor_workflows($1)"#,
        )
        .bind(executor_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn start_move(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.start_workflow_run_move($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        match self.read_one(workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn complete_move(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.complete_workflow_run_move($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        match self.read_one(workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn scheduled_listener(
        &self,
        executor_id: &ExecutorId,
    ) -> EmResult<Self::ScheduledListener> {
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener
            .listen(&format!("wr_scheduled_{}", executor_id))
            .await?;
        Ok(PgChangeListener::new(listener))
    }

    async fn cancel_listener(&self, executor_id: &ExecutorId) -> EmResult<Self::CancelListener> {
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener
            .listen(&format!("wr_canceled_{}", executor_id))
            .await?;
        Ok(PgChangeListener::new(listener))
    }
}
