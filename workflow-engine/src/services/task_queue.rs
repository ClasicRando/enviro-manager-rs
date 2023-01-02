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

use crate::database::finish_transaction;

use super::error::ServiceResult;

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

#[derive(sqlx::FromRow, Serialize, Deserialize)]
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

struct TaskQueueService {
    pool: &'static PgPool,
}

impl TaskQueueService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn read_one(
        &self,
        request: TaskQueueRequest,
    ) -> ServiceResult<Option<TaskQueueRecord>> {
        let result = sqlx::query_as(
            r#"
            select workflow_run_id, task_order, task_id, parameters, url
            from   task_queue
            where  workflow_run_id = $1
            and    task_order = $2"#,
        )
        .bind(request.workflow_run_id)
        .bind(request.task_order)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn retry_task(&self, request: TaskQueueRequest) -> ServiceResult<()> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call workflow_engine.retry_task($1, $2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        Ok(())
    }

    pub async fn complete_task(&self, request: TaskQueueRequest) -> ServiceResult<()> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call workflow_engine.complete_task($1, $2)")
            .bind(request.workflow_run_id)
            .bind(request.task_order)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        Ok(())
    }
}
