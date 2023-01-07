use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    PgPool,
};

use crate::{database::finish_transaction, error::Result as WEResult};

#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "workflow_task")]
pub struct WorkflowTask {
    task_order: i32,
    task_id: i64,
    name: String,
    description: String,
    parameters: Option<Value>,
    service_name: String,
    url: String,
}

impl PgHasArrayType for WorkflowTask {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_task")
    }
}

#[derive(sqlx::Type, Deserialize)]
#[sqlx(type_name = "workflow_task_request")]
pub struct WorkflowTaskRequest {
    task_id: i64,
    parameters: Option<Value>,
}

impl PgHasArrayType for WorkflowTaskRequest {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_task_request")
    }
}

#[derive(Deserialize)]
pub struct WorkflowRequest {
    name: String,
    tasks: Vec<WorkflowTaskRequest>,
}

#[derive(Deserialize)]
pub struct WorkflowDeprecationRequest {
    workflow_id: i64,
    new_workflow_id: Option<i64>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Workflow {
    workflow_id: i64,
    name: String,
    tasks: Vec<WorkflowTask>,
}

pub struct WorkflowsService {
    pool: &'static PgPool,
}

impl WorkflowsService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: WorkflowRequest) -> WEResult<Workflow> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query_scalar("select create_workflow($1,$2)")
            .bind(request.name)
            .bind(request.tasks)
            .fetch_one(&mut transaction)
            .await;
        let workflow_id: i64 = finish_transaction(transaction, result).await?;
        match self.read_one(workflow_id).await {
            Ok(Some(workflow)) => Ok(workflow),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    pub async fn read_one(&self, workflow_id: i64) -> WEResult<Option<Workflow>> {
        let result = sqlx::query_as(
            r#"
            select workflow_id, name, tasks
            from   v_workflows
            where  workflow_id = $1"#,
        )
        .bind(workflow_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> WEResult<Vec<Workflow>> {
        let result = sqlx::query_as(
            r#"
            select workflow_id, name, tasks
            from   v_workflows"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn deprecate(&self, request: WorkflowDeprecationRequest) -> WEResult<i64> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call deprecate_workflow($1,$2)")
            .bind(request.workflow_id)
            .bind(request.new_workflow_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        Ok(request.workflow_id)
    }
}
