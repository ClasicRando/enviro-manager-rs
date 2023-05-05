use common::error::{EmError, EmResult};
use futures::StreamExt;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef,
    },
    Database, PgPool, Pool, Postgres, Type,
};

use super::workflow_runs::WorkflowRunId;

/// Check performed during a task run to validate the current state of a task or the system that the
/// task is operating on. Rules must always have a non-empty and unique `name` per task, as well as
/// a `failed` status and optional `message` to provide details of what the rule checked.
///
/// Since the `message` field is optional, the Type trait must be manually derived to encode and
/// decode from a Postgres database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRule {
    name: String,
    failed: bool,
    message: Option<String>,
}

impl Encode<'_, Postgres> for TaskRule {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);
        encoder.encode(&self.name);
        encoder.encode(self.failed);
        encoder.encode(&self.message);
        encoder.finish();
        IsNull::No
    }

    fn size_hint(&self) -> usize {
        3usize * (4 + 4)
            + <String as Encode<Postgres>>::size_hint(&self.name)
            + <bool as Encode<Postgres>>::size_hint(&self.failed)
            + <Option<String> as Encode<Postgres>>::size_hint(&self.message)
    }
}

impl<'r> Decode<'r, Postgres> for TaskRule {
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let mut decoder = PgRecordDecoder::new(value)?;
        let name: String = decoder.try_decode()?;
        let failed: bool = decoder.try_decode()?;
        let message: Option<String> = decoder.try_decode()?;
        Ok(TaskRule {
            name,
            failed,
            message,
        })
    }
}

impl Type<Postgres> for TaskRule {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("task_rule")
    }
}

impl PgHasArrayType for TaskRule {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_task_rule")
    }
}

/// Represents a row from the `task.task_queue` table
#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct TaskQueueRecord {
    workflow_run_id: i64,
    task_order: i32,
    task_id: i64,
    parameters: Option<Value>,
    url: String,
}

/// Container for the data required to fetch/update a single `task.task_queue` record
#[derive(Deserialize)]
pub struct TaskQueueRequest {
    workflow_run_id: i64,
    task_order: i32,
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

#[async_trait::async_trait]
pub trait TaskQueueService: Clone + Send + Sync + 'static {
    type Database: Database;

    /// Create a new [TaskQueueService] with the referenced pool as the data source
    fn new(pool: &Pool<Self::Database>) -> Self;
    /// Read a single task record from `task.task_queue` for the specified `request`data. Will
    /// return [None] when the ids in the `request` do not match a record.
    async fn read_one(&self, request: TaskQueueRequest) -> EmResult<Option<TaskQueueRecord>>;
    /// Append the task `rule` data to the specified `task_queue` record
    async fn append_task_rule(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: i32,
        rule: TaskRule,
    ) -> EmResult<()>;
    /// Update the specified `task_queue` record with the new progress value
    async fn set_task_progress(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: i32,
        progress: i16,
    ) -> EmResult<()>;
    /// Retry the specified `task_queue` record. Note, the record must be in the 'Failed' or
    /// 'Rule Broken' state to qualify for a retry.
    async fn retry_task(&self, request: TaskQueueRequest) -> EmResult<()>;
    /// Complete the specified `task_queue` record to allow for continuing of a workflow run after
    /// a user interruption. Note, the record must be in the 'Paused' state for a successful
    /// complete.
    async fn complete_task(&self, request: TaskQueueRequest) -> EmResult<()>;
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

/// Service for fetching and interacting with `task_queue` data. Wraps a [PgPool] and provides
/// interaction methods for the API and [Executor][crate::executor::Executor] instances.
#[derive(Clone)]
pub struct PgTaskQueueService {
    pool: PgPool,
}

impl PgTaskQueueService {
    /// Process a response `message` from a remote task run. The expected format is of MessagePack
    /// and the contents are parsed to a [TaskResponse] variant. If the message is a
    /// [TaskResponse::Done] message, the contents are returned as a tuple. Otherwise, a [None]
    /// value is returned to signify the message has been processed but there are more to come.
    async fn process_response_message(
        &self,
        message: &[u8],
        workflow_run_id: &WorkflowRunId,
        record: &TaskQueueRecord,
    ) -> EmResult<Option<(bool, Option<String>)>> {
        match rmp_serde::from_slice(message)? {
            TaskResponse::Progress(progress) => {
                self.set_task_progress(workflow_run_id, record.task_order, progress)
                    .await?;
            }
            TaskResponse::Rule(rule) => {
                self.append_task_rule(workflow_run_id, record.task_order, rule)
                    .await?
            }
            TaskResponse::Done { success, message } => return Ok(Some((success, message))),
        }
        Ok(None)
    }

    /// Execute a remove task for the specified task `record`. Creates a new [Client] and proceeds
    /// to make a POST request against the specified task url with the `record` as a serialized
    /// MessagePack body. The result of the request is interpreted as a byte stream and
    /// [TaskResponse] messages are parsed from it until a [TaskResponse::Done] message is sent. If
    /// the stream ends without a [TaskResponse::Done] message, a [ExitedTask][EmError::ExitedTask]
    /// error is returned.
    async fn remote_task_run(&self, record: &TaskQueueRecord) -> EmResult<(bool, Option<String>)> {
        let workflow_run_id = record.workflow_run_id.into();
        let client = Client::new();
        let buffer = rmp_serde::to_vec(record)?;
        let mut stream = client
            .request(Method::POST, &record.url)
            .body(buffer)
            .send()
            .await?
            .bytes_stream();
        while let Some(chunk) = stream.next().await {
            let message = match chunk {
                Ok(message) => message,
                Err(error) => return Err(error.into()),
            };
            let message = self
                .process_response_message(&message, &workflow_run_id, record)
                .await?;
            if let Some(done_message) = message {
                return Ok(done_message);
            }
        }
        Err(EmError::ExitedTask)
    }
}

#[async_trait::async_trait]
impl TaskQueueService for PgTaskQueueService {
    type Database = Postgres;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn read_one(&self, request: TaskQueueRequest) -> EmResult<Option<TaskQueueRecord>> {
        let result = sqlx::query_as(
            r#"
            select tq.workflow_run_id, tq.task_order, tq.task_id, tq.parameters, tq.url
            from task.task_queue tq
            where
                tq.workflow_run_id = $1
                and tq.task_order = $2"#,
        )
        .bind(request.workflow_run_id)
        .bind(request.task_order)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    async fn append_task_rule(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: i32,
        rule: TaskRule,
    ) -> EmResult<()> {
        sqlx::query("call workflow.append_task_rule($1,$2,$3)")
            .bind(workflow_run_id)
            .bind(task_order)
            .bind(rule)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_task_progress(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: i32,
        progress: i16,
    ) -> EmResult<()> {
        sqlx::query("call workflow.set_task_progress($1,$2,$3)")
            .bind(workflow_run_id)
            .bind(task_order)
            .bind(progress)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn retry_task(&self, request: TaskQueueRequest) -> EmResult<()> {
        sqlx::query("call workflow.retry_task($1,$2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn complete_task(&self, request: TaskQueueRequest) -> EmResult<()> {
        sqlx::query("call workflow.complete_task($1,$2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn next_task(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> EmResult<Option<TaskQueueRecord>> {
        let task_queue_record = sqlx::query_as("call workflow.acquire_next_task($1)")
            .bind(workflow_run_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(task_queue_record)
    }

    async fn run_task(&self, record: &TaskQueueRecord) -> EmResult<(bool, Option<String>)> {
        self.pool
            .close_event()
            .do_until(self.remote_task_run(record))
            .await?
    }

    async fn fail_task_run(&self, record: &TaskQueueRecord, error: EmError) -> EmResult<()> {
        sqlx::query("call fail_task_run($1,$2,$3)")
            .bind(record.workflow_run_id)
            .bind(record.task_order)
            .bind(error.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn complete_task_run(
        &self,
        record: &TaskQueueRecord,
        is_paused: bool,
        message: Option<String>,
    ) -> EmResult<()> {
        sqlx::query("call complete_task_run($1,$2,$3,$4)")
            .bind(record.workflow_run_id)
            .bind(record.task_order)
            .bind(is_paused)
            .bind(message)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
