use std::str::FromStr;

use chrono::NaiveDateTime;
use rocket::request::FromParam;
use serde::Serialize;
use serde_json::Value;
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgListener, PgTypeInfo, PgValueRef,
    },
    PgPool, Postgres, Type,
};

use crate::error::{Error as WEError, Result as WEResult};

use super::{executors::ExecutorId, task_queue::TaskRule, tasks::TaskStatus};

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

#[derive(sqlx::FromRow, Serialize)]
pub struct WorkflowRun {
    workflow_run_id: i64,
    workflow_id: i64,
    status: WorkflowRunStatus,
    executor_id: Option<i64>,
    progress: i16,
    tasks: Vec<WorkflowRunTask>,
}

#[derive(sqlx::FromRow)]
pub struct ExecutorWorkflowRun {
    pub workflow_run_id: WorkflowRunId,
    pub status: WorkflowRunStatus,
    pub is_valid: bool,
}

#[derive(sqlx::Type, Eq, PartialEq, Hash, Clone)]
#[sqlx(transparent)]
pub struct WorkflowRunId(i64);

impl FromStr for WorkflowRunId {
    type Err = WEError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<i64>()?.into())
    }
}

impl From<i64> for WorkflowRunId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl<'a> FromParam<'a> for WorkflowRunId {
    type Error = WEError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        param.parse()
    }
}

impl std::fmt::Display for WorkflowRunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct WorkflowRunsService {
    pool: &'static PgPool,
}

impl WorkflowRunsService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn initialize(&self, workflow_id: i64) -> WEResult<WorkflowRun> {
        let workflow_run_id = sqlx::query_scalar("select initialize_workflow_run($1)")
            .bind(workflow_id)
            .fetch_one(self.pool)
            .await?;
        match self.read_one(&workflow_run_id).await {
            Ok(Some(workflow_run)) => Ok(workflow_run),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    pub async fn read_one(&self, workflow_run_id: &WorkflowRunId) -> WEResult<Option<WorkflowRun>> {
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

    pub async fn read_many(&self) -> WEResult<Vec<WorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select workflow_run_id, workflow_id, status, executor_id, progress, tasks
            from   v_workflow_runs"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn cancel(&self, workflow_run_id: &WorkflowRunId) -> WEResult<Option<WorkflowRun>> {
        sqlx::query("call cancel_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn schedule(&self, workflow_run_id: &WorkflowRunId) -> WEResult<Option<WorkflowRun>> {
        sqlx::query("call schedule_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn schedule_with_executor(
        &self,
        workflow_run_id: &WorkflowRunId,
        executor_id: &ExecutorId,
    ) -> WEResult<Option<WorkflowRun>> {
        sqlx::query("call schedule_workflow_run($1,$2)")
            .bind(workflow_run_id)
            .bind(executor_id)
            .execute(self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn restart(&self, workflow_run_id: &WorkflowRunId) -> WEResult<Option<WorkflowRun>> {
        sqlx::query("call restart_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn complete(&self, workflow_run_id: &WorkflowRunId) -> WEResult<()> {
        sqlx::query("call complete_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn all_executor_workflows(
        &self,
        executor_id: &ExecutorId,
    ) -> WEResult<Vec<ExecutorWorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select workflow_run_id, status, is_valid
            from   all_executor_workflows($1)"#,
        )
        .bind(executor_id)
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn start_move(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> WEResult<Option<WorkflowRun>> {
        sqlx::query("call start_workflow_run_move($1)")
            .bind(workflow_run_id)
            .execute(self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn complete_move(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> WEResult<Option<WorkflowRun>> {
        sqlx::query("call complete_workflow_run_move($1)")
            .bind(workflow_run_id)
            .execute(self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    pub async fn scheduled_listener(&self, executor_id: &ExecutorId) -> WEResult<PgListener> {
        let mut listener = PgListener::connect_with(self.pool).await?;
        listener
            .listen(&format!("wr_scheduled_{}", executor_id))
            .await?;
        Ok(listener)
    }

    pub async fn cancel_listener(&self, executor_id: &ExecutorId) -> WEResult<PgListener> {
        let mut listener = PgListener::connect_with(self.pool).await?;
        listener
            .listen(&format!("wr_canceled_{}", executor_id))
            .await?;
        Ok(listener)
    }
}
