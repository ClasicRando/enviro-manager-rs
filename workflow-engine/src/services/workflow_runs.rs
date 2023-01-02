use chrono::NaiveDateTime;
use serde::Serialize;
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

use crate::database::finish_transaction;

use super::{error::ServiceResult, task_queue::TaskRule, tasks::TaskStatus};

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

#[derive(sqlx::FromRow, Serialize)]
pub struct WorkflowRun {
    workflow_run_id: i64,
    workflow_id: i64,
    status: WorkflowRunStatus,
    executor_id: Option<i64>,
    progress: i16,
    tasks: Vec<WorkflowRunTask>,
}

struct WorkflowRunsService {
    pool: &'static PgPool,
}

impl WorkflowRunsService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn initialize(&self, workflow_id: i64) -> ServiceResult<WorkflowRun> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query_scalar("select initialize_workflow_run($1)")
            .bind(workflow_id)
            .fetch_one(&mut transaction)
            .await;
        let workflow_run_id: i64 = finish_transaction(transaction, result).await?;
        match self.read_one(workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    pub async fn read_one(&self, workflow_run_id: i64) -> ServiceResult<Option<WorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select workflow_run_id, workflow_id, status, executor_id, progress, tasks
            from   v_workflow_runs
            where  workflow_run_id = $1"#,
        )
        .bind(workflow_run_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> ServiceResult<Vec<WorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select workflow_run_id, workflow_id, status, executor_id, progress, tasks
            from   v_workflow_runs"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn cancel(&self, workflow_run_id: i64) -> ServiceResult<Option<WorkflowRun>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call cancel_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn schedule(&self, workflow_run_id: i64) -> ServiceResult<Option<WorkflowRun>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call schedule_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn restart(&self, workflow_run_id: i64) -> ServiceResult<Option<WorkflowRun>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call restart_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(workflow_run_id).await
    }
}
