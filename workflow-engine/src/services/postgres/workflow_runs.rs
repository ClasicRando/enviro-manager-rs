use chrono::NaiveDateTime;
use common::{
    database::postgres::{listener::PgChangeListener, Postgres},
    error::{EmError, EmResult},
};
use serde_json::Value;
use sqlx::{
    encode::IsNull,
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgListener, PgTypeInfo, PgValueRef,
    },
    Decode, Encode, PgPool, Type,
};

use crate::{
    executor::utilities::{WorkflowRunCancelMessage, WorkflowRunScheduledMessage},
    services::{
        executors::ExecutorId,
        task_queue::{TaskRule, TaskStatus},
        workflow_runs::{ExecutorWorkflowRun, WorkflowRun, WorkflowRunId, WorkflowRunTask},
        workflows::WorkflowId,
    },
    WorkflowRunsService,
};

impl Encode<'_, sqlx::Postgres> for WorkflowRunTask {
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
            + <i32 as Encode<sqlx::Postgres>>::size_hint(&self.task_order)
            + <i64 as Encode<sqlx::Postgres>>::size_hint(&self.task_id)
            + <String as Encode<sqlx::Postgres>>::size_hint(&self.name)
            + <String as Encode<sqlx::Postgres>>::size_hint(&self.description)
            + <TaskStatus as Encode<sqlx::Postgres>>::size_hint(&self.task_status)
            + <Option<Value> as Encode<sqlx::Postgres>>::size_hint(&self.parameters)
            + <Option<String> as Encode<sqlx::Postgres>>::size_hint(&self.output)
            + <Option<Vec<TaskRule>> as Encode<sqlx::Postgres>>::size_hint(&self.rules)
            + <Option<NaiveDateTime> as Encode<sqlx::Postgres>>::size_hint(&self.task_start)
            + <Option<NaiveDateTime> as Encode<sqlx::Postgres>>::size_hint(&self.task_end)
            + <Option<i16> as Encode<sqlx::Postgres>>::size_hint(&self.progress)
    }
}

impl<'r> Decode<'r, sqlx::Postgres> for WorkflowRunTask {
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

impl Type<sqlx::Postgres> for WorkflowRunTask {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("workflow_run_task")
    }
}

impl PgHasArrayType for WorkflowRunTask {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_run_task")
    }
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

    fn create(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn initialize(&self, workflow_id: &WorkflowId) -> EmResult<WorkflowRun> {
        let workflow_run_id = sqlx::query_scalar("select workflow.initialize_workflow_run($1)")
            .bind(workflow_id)
            .fetch_one(&self.pool)
            .await?;
        self.read_one(&workflow_run_id).await
    }

    async fn read_one(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
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
        match result {
            Some(workflow_run) => Ok(workflow_run),
            None => Err(EmError::MissingRecord {
                pk: workflow_run_id.to_string(),
            }),
        }
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
        self.read_one(workflow_run_id).await
    }

    async fn schedule(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.schedule_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        self.read_one(workflow_run_id).await
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
        self.read_one(workflow_run_id).await
    }

    async fn restart(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.restart_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    async fn update_progress(&self, workflow_run_id: &WorkflowRunId) -> EmResult<()> {
        sqlx::query("call workflow.update_progress($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
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
            from executor.executor_workflows($1)"#,
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
        self.read_one(workflow_run_id).await
    }

    async fn complete_move(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow.complete_workflow_run_move($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        self.read_one(workflow_run_id).await
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
