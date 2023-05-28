use common::{
    database::connection::finalize_transaction,
    error::{EmError, EmResult},
};
use futures::StreamExt;
use reqwest::{Client, Method};
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef,
    },
    PgPool, Postgres, Type,
};

use crate::{
    services::{
        postgres::workflow_runs::PgWorkflowRunsService,
        task_queue::{TaskQueueRequest, TaskRule, TaskStatus},
        workflow_runs::WorkflowRunId,
    },
    TaskQueueRecord, TaskQueueService, TaskResponse, WorkflowRunsService,
};

impl Encode<'_, Postgres> for TaskRule {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);
        encoder.encode(&self.name);
        encoder.encode(&self.failed);
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

/// Postgres implementation of TaskQueueService
#[derive(Clone)]
pub struct PgTaskQueueService {
    pool: PgPool,
    workflow_runs_service: PgWorkflowRunsService,
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
                self.set_task_progress(workflow_run_id, &record.task_order, progress)
                    .await?;
            }
            TaskResponse::Rule(rule) => {
                self.append_task_rule(workflow_run_id, &record.task_order, rule)
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
                .process_response_message(&message, &record.workflow_run_id, record)
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
    type WorkflowRunService = PgWorkflowRunsService;

    fn create(pool: &PgPool, workflow_runs_service: &PgWorkflowRunsService) -> Self {
        Self {
            pool: pool.clone(),
            workflow_runs_service: workflow_runs_service.clone(),
        }
    }

    async fn read_one(&self, request: &TaskQueueRequest) -> EmResult<TaskQueueRecord> {
        let result = sqlx::query_as(
            r#"
            select tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, tq.url
            from task.v_task_queue_record tq
            where
                tq.workflow_run_id = $1
                and tq.task_order = $2"#,
        )
        .bind(&request.workflow_run_id)
        .bind(&request.task_order)
        .fetch_optional(&self.pool)
        .await?;
        match result {
            Some(record) => Ok(record),
            None => Err(EmError::MissingRecord {
                pk: format!("{} + {}", request.workflow_run_id, request.task_order),
            }),
        }
    }

    async fn append_task_rule(
        &self,
        workflow_run_id: &WorkflowRunId,
        task_order: &i32,
        rule: TaskRule,
    ) -> EmResult<()> {
        sqlx::query("call task.append_task_rule($1,$2,$3)")
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
        task_order: &i32,
        progress: i16,
    ) -> EmResult<()> {
        sqlx::query("call task.set_task_progress($1,$2,$3)")
            .bind(workflow_run_id)
            .bind(task_order)
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

        let retry_result = sqlx::query("call task.retry_task($1,$2)")
            .bind(&request.workflow_run_id)
            .bind(&request.task_order)
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
            return Err(error.into());
        }

        transaction.commit().await?;
        Ok(())
    }

    async fn complete_task(&self, request: &TaskQueueRequest) -> EmResult<()> {
        let mut transaction = self.pool.begin().await?;
        let complete_task_result = sqlx::query("call task.complete_task($1,$2)")
            .bind(&request.workflow_run_id)
            .bind(&request.task_order)
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
            from task.next_task($1) nt
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

        let start_task_result = sqlx::query("call task.start_task_run($1,$2)")
            .bind(&task_queue_record.workflow_run_id)
            .bind(&task_queue_record.task_order)
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
        sqlx::query("call task.fail_task_run($1,$2,$3)")
            .bind(&record.workflow_run_id)
            .bind(&record.task_order)
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
        let complete_result = sqlx::query("call task.complete_task_run($1,$2,$3,$4)")
            .bind(&record.workflow_run_id)
            .bind(&record.task_order)
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
