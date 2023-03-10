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
    PgPool, Postgres, Type,
};

use super::workflow_runs::WorkflowRunId;

use crate::error::{Error as WEError, Result as WEResult};

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

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct TaskQueueRecord {
    workflow_run_id: i64,
    task_order: i32,
    task_id: i64,
    parameters: Option<Value>,
    url: String,
}

#[derive(Deserialize)]
pub struct TaskQueueRequest {
    workflow_run_id: i64,
    task_order: i32,
}

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

pub struct TaskQueueService {
    pool: &'static PgPool,
}

impl TaskQueueService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn read_one(&self, request: TaskQueueRequest) -> WEResult<Option<TaskQueueRecord>> {
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
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    async fn append_task_rule(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: i32,
        rule: TaskRule,
    ) -> WEResult<()> {
        sqlx::query("call append_task_rule($1,$2,$3)")
            .bind(workflow_run_id)
            .bind(task_order)
            .bind(rule)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    async fn set_task_progress(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: i32,
        progress: i16,
    ) -> WEResult<()> {
        sqlx::query("call set_task_progress($1,$2,$3)")
            .bind(workflow_run_id)
            .bind(task_order)
            .bind(progress)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn retry_task(&self, request: TaskQueueRequest) -> WEResult<()> {
        sqlx::query("call retry_task($1,$2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn complete_task(&self, request: TaskQueueRequest) -> WEResult<()> {
        sqlx::query("call complete_task($1,$2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn next_task(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> WEResult<Option<TaskQueueRecord>> {
        let task_queue_record = sqlx::query_as("call workflow.acquire_next_task($1)")
            .bind(workflow_run_id)
            .fetch_optional(self.pool)
            .await?;
        Ok(task_queue_record)
    }

    async fn process_response_message(
        &self,
        message: &[u8],
        workflow_run_id: &WorkflowRunId,
        record: &TaskQueueRecord,
    ) -> WEResult<Option<(bool, Option<String>)>> {
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

    async fn remote_task_run(&self, record: &TaskQueueRecord) -> WEResult<(bool, Option<String>)> {
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
        Err(WEError::ExitedTask)
    }

    pub async fn run_task(&self, record: &TaskQueueRecord) -> WEResult<(bool, Option<String>)> {
        let result = self
            .pool
            .close_event()
            .do_until(self.remote_task_run(record))
            .await?;
        result
    }

    pub async fn fail_task_run(&self, record: &TaskQueueRecord, error: WEError) -> WEResult<()> {
        sqlx::query("call fail_task_run($1,$2,$3)")
            .bind(record.workflow_run_id)
            .bind(record.task_order)
            .bind(error.to_string())
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn complete_task_run(
        &self,
        record: &TaskQueueRecord,
        is_paused: bool,
        message: Option<String>,
    ) -> WEResult<()> {
        sqlx::query("call complete_task_run($1,$2,$3,$4)")
            .bind(record.workflow_run_id)
            .bind(record.task_order)
            .bind(is_paused)
            .bind(message)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
