use chrono::NaiveDateTime;
use common::{
    database::{
        connection::finalize_transaction,
        postgres::{listener::PgChangeListener, Postgres},
    },
    error::{EmError, EmResult},
};
use futures::StreamExt;
use reqwest::{Client, Method};
use serde_json::Value;
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgListener, PgTypeInfo, PgValueRef,
    },
    PgPool, Type,
};

use crate::{
    executor::{
        data::ExecutorId,
        utilities::{WorkflowRunCancelMessage, WorkflowRunScheduledMessage},
    },
    workflow::{
        data::{TaskId, WorkflowId},
        service::{postgres::PgWorkflowsService, WorkflowsService},
    },
    workflow_run::{
        data::{
            ExecutorWorkflowRun, TaskQueueRecord, TaskQueueRequest, TaskResponse, TaskRule,
            TaskStatus, WorkflowRun, WorkflowRunId, WorkflowRunTask,
        },
        service::{TaskQueueService, WorkflowRunsService},
    },
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
            + <TaskId as Encode<sqlx::Postgres>>::size_hint(&self.task_id)
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
        let task_id = decoder.try_decode::<TaskId>()?;
        let name = decoder.try_decode::<String>()?;
        let description = decoder.try_decode::<String>()?;
        let task_status = decoder.try_decode::<TaskStatus>()?;
        let parameters = decoder.try_decode::<Option<Value>>()?;
        let output = decoder.try_decode::<Option<String>>()?;
        let rules = decoder.try_decode::<Option<Vec<TaskRule>>>()?;
        let task_start = decoder.try_decode::<Option<NaiveDateTime>>()?;
        let task_end = decoder.try_decode::<Option<NaiveDateTime>>()?;
        let progress = decoder.try_decode::<Option<i16>>()?;
        Ok(Self {
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
    workflow_service: PgWorkflowsService,
}

impl PgWorkflowRunsService {
    /// Create a new [PgWorkflowRunsService] with the referenced pool as the data source
    pub fn new(pool: &PgPool, workflow_service: &PgWorkflowsService) -> Self {
        Self {
            pool: pool.clone(),
            workflow_service: workflow_service.clone(),
        }
    }
}

#[async_trait::async_trait]
impl WorkflowRunsService for PgWorkflowRunsService {
    type CancelListener = PgChangeListener<WorkflowRunCancelMessage>;
    type Database = Postgres;
    type ScheduledListener = PgChangeListener<WorkflowRunScheduledMessage>;
    type WorkflowService = PgWorkflowsService;

    async fn initialize(&self, workflow_id: &WorkflowId) -> EmResult<WorkflowRun> {
        let workflow = self.workflow_service.read_one(workflow_id).await?;
        if workflow.is_deprecated {
            return Err(EmError::Generic(format!(
                "Cannot initialize a workflow_run with a deprecated workflow. Consider using \
                 workflow_id = {:?}",
                workflow.new_workflow
            )));
        }

        let workflow_run_id =
            sqlx::query_scalar("call workflow_run.initialize_workflow_run($1,null)")
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
            from workflow_run.v_workflow_runs wr
            where wr.workflow_run_id = $1"#,
        )
        .bind(workflow_run_id)
        .fetch_optional(&self.pool)
        .await?;
        result.map_or_else(
            || {
                Err(EmError::MissingRecord {
                    pk: workflow_run_id.to_string(),
                })
            },
            Ok,
        )
    }

    async fn read_many(&self) -> EmResult<Vec<WorkflowRun>> {
        let result = sqlx::query_as(
            r#"
            select
                wr.workflow_run_id, wr.workflow_id, wr.status, wr.executor_id, wr.progress,
                wr.tasks
            from workflow_run.v_workflow_runs wr"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn cancel(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow_run.cancel_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    async fn schedule(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow_run.schedule_workflow_run($1)")
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
        sqlx::query("call workflow_run.schedule_workflow_run($1,$2)")
            .bind(workflow_run_id)
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    async fn restart(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow_run.restart_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    async fn update_progress(&self, workflow_run_id: &WorkflowRunId) -> EmResult<()> {
        sqlx::query("call workflow_run.update_progress($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn complete(&self, workflow_run_id: &WorkflowRunId) -> EmResult<()> {
        sqlx::query("call workflow_run.complete_workflow_run($1)")
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
            from workflow_run.executor_workflows($1)"#,
        )
        .bind(executor_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn start_move(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow_run.start_workflow_run_move($1)")
            .bind(workflow_run_id)
            .execute(&self.pool)
            .await?;
        self.read_one(workflow_run_id).await
    }

    async fn complete_move(&self, workflow_run_id: &WorkflowRunId) -> EmResult<WorkflowRun> {
        sqlx::query("call workflow_run.complete_workflow_run_move($1)")
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

impl Encode<'_, sqlx::Postgres> for TaskRule {
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
            + <String as Encode<sqlx::Postgres>>::size_hint(&self.name)
            + <bool as Encode<sqlx::Postgres>>::size_hint(&self.failed)
            + <Option<String> as Encode<sqlx::Postgres>>::size_hint(&self.message)
    }
}

impl<'r> Decode<'r, sqlx::Postgres> for TaskRule {
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let mut decoder = PgRecordDecoder::new(value)?;
        let name: String = decoder.try_decode()?;
        let failed: bool = decoder.try_decode()?;
        let message: Option<String> = decoder.try_decode()?;
        Ok(Self {
            name,
            failed,
            message,
        })
    }
}

impl Type<sqlx::Postgres> for TaskRule {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("task_rule")
    }
}

impl PgHasArrayType for TaskRule {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_task_rule")
    }
}

/// Postgres implementation of TaskQueueService
#[derive(Clone)]
pub struct PgTaskQueueService {
    pool: PgPool,
    workflow_runs_service: PgWorkflowRunsService,
}

impl PgTaskQueueService {
    /// Create a new [PgTaskQueueService] with the referenced pool as the data source
    pub fn new(pool: &PgPool, workflow_runs_service: &PgWorkflowRunsService) -> Self {
        Self {
            pool: pool.clone(),
            workflow_runs_service: workflow_runs_service.clone(),
        }
    }

    /// Process a response `message` from a remote task run. The expected format is of MessagePack
    /// and the contents are parsed to a [TaskResponse] variant. If the message is a
    /// [TaskResponse::Done] message, the contents are returned as a tuple. Otherwise, a [None]
    /// value is returned to signify the message has been processed but there are more to come.
    async fn process_response_message(
        &self,
        message: &[u8],
        record: &TaskQueueRecord,
    ) -> EmResult<Option<(bool, Option<String>)>> {
        match rmp_serde::from_slice(message)? {
            TaskResponse::Progress(progress) => {
                let request = TaskQueueRequest {
                    workflow_run_id: record.workflow_run_id,
                    task_order: record.task_order,
                };
                self.set_task_progress(&request, progress).await?;
            }
            TaskResponse::Rule(rule) => {
                let request = TaskQueueRequest {
                    workflow_run_id: record.workflow_run_id,
                    task_order: record.task_order,
                };
                self.append_task_rule(&request, &rule).await?
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
            let message = self.process_response_message(&message, record).await?;
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
    type WorkflowRunService = PgWorkflowRunsService;

    async fn read_one(&self, request: &TaskQueueRequest) -> EmResult<TaskQueueRecord> {
        let result = sqlx::query_as(
            r#"
            select tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, tq.url
            from workflow_run.v_task_queue_record tq
            where
                tq.workflow_run_id = $1
                and tq.task_order = $2"#,
        )
        .bind(request.workflow_run_id)
        .bind(request.task_order)
        .fetch_optional(&self.pool)
        .await?;
        result.map_or_else(
            || {
                Err(EmError::MissingRecord {
                    pk: format!("{} + {}", request.workflow_run_id, request.task_order),
                })
            },
            Ok,
        )
    }

    async fn append_task_rule(&self, request: &TaskQueueRequest, rule: &TaskRule) -> EmResult<()> {
        if rule.name.trim().is_empty() {
            return Err("Task rule attribute 'name' cannot be empty or whitespace".into());
        }
        sqlx::query("call workflow_run.append_task_rule($1,$2,$3)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .bind(rule)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_task_progress(&self, request: &TaskQueueRequest, progress: i16) -> EmResult<()> {
        sqlx::query("call workflow_run.set_task_progress($1,$2,$3)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .bind(progress)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn retry_task(&self, request: &TaskQueueRequest) -> EmResult<()> {
        let task_queue_record = self.read_one(request).await?;
        if task_queue_record.status != TaskStatus::RuleBroken
            && task_queue_record.status != TaskStatus::Failed
        {
            return Err(EmError::Generic(
                "Cannot retry task. Status must be 'Failed' or 'Rule Broken'".to_owned(),
            ));
        }

        let mut transaction = self.pool.begin().await?;

        let retry_result = sqlx::query("call workflow_run.retry_task($1,$2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(&mut transaction)
            .await;
        if let Err(error) = retry_result {
            transaction.rollback().await?;
            return Err(error.into());
        }

        if let Err(error) = self
            .workflow_runs_service
            .schedule(&request.workflow_run_id)
            .await
        {
            transaction.rollback().await?;
            return Err(error);
        }

        transaction.commit().await?;
        Ok(())
    }

    async fn complete_task(&self, request: &TaskQueueRequest) -> EmResult<()> {
        let mut transaction = self.pool.begin().await?;
        let complete_task_result = sqlx::query("call workflow_run.complete_task($1,$2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(&mut transaction)
            .await;
        if let Err(error) = complete_task_result {
            transaction.rollback().await?;
            return Err(error.into());
        }

        if let Err(error) = self
            .workflow_runs_service
            .update_progress(&request.workflow_run_id)
            .await
        {
            transaction.rollback().await?;
            return Err(error);
        }

        if let Err(error) = self
            .workflow_runs_service
            .schedule(&request.workflow_run_id)
            .await
        {
            transaction.rollback().await?;
            return Err(error);
        }

        transaction.commit().await?;
        Ok(())
    }

    async fn next_task(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> EmResult<Option<TaskQueueRecord>> {
        let mut transaction = self.pool.begin().await?;
        let fetch_result = sqlx::query_as(
            r#"
            select nt.workflow_run_id, nt.task_order, nt.task_id, nt.status, nt.parameters, nt.url
            from workflow_run.next_task($1) nt
            where nt.task_order is not null"#,
        )
        .bind(workflow_run_id)
        .fetch_optional(&mut transaction)
        .await;

        let task_queue_record: TaskQueueRecord = match fetch_result {
            Ok(Some(inner)) => inner,
            Ok(None) => {
                transaction.commit().await?;
                return Ok(None);
            }
            Err(error) => {
                transaction.rollback().await?;
                return Err(error.into());
            }
        };

        let start_task_result = sqlx::query("call workflow_run.start_task_run($1,$2)")
            .bind(task_queue_record.workflow_run_id)
            .bind(task_queue_record.task_order)
            .execute(&mut transaction)
            .await;

        finalize_transaction(start_task_result, transaction).await?;
        Ok(Some(task_queue_record))
    }

    async fn run_task(&self, record: &TaskQueueRecord) -> EmResult<(bool, Option<String>)> {
        self.pool
            .close_event()
            .do_until(self.remote_task_run(record))
            .await?
    }

    async fn fail_task_run(&self, record: &TaskQueueRecord, error: EmError) -> EmResult<()> {
        sqlx::query("call workflow_run.fail_task_run($1,$2,$3)")
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
        let mut transaction = self.pool.begin().await?;
        let complete_result = sqlx::query("call workflow_run.complete_task_run($1,$2,$3,$4)")
            .bind(record.workflow_run_id)
            .bind(record.task_order)
            .bind(is_paused)
            .bind(message)
            .execute(&mut transaction)
            .await;
        if let Err(error) = complete_result {
            transaction.rollback().await?;
            return Err(error.into());
        }

        if let Err(error) = self
            .workflow_runs_service
            .update_progress(&record.workflow_run_id)
            .await
        {
            transaction.rollback().await?;
            return Err(error);
        }

        transaction.commit().await?;
        Ok(())
    }
}
