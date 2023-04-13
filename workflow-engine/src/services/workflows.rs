use rocket::request::FromParam;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    PgPool,
};

use crate::error::{Error as WEError, Result as WEResult};

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

#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct WorkflowId(i64);

impl From<i64> for WorkflowId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl<'a> FromParam<'a> for WorkflowId {
    type Error = WEError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(param.parse::<i64>()?))
    }
}

impl std::fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct WorkflowsService {
    pool: PgPool,
}

impl WorkflowsService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    // TODO: Alter create_workflow to return Workflow data
    pub async fn create(&self, request: WorkflowRequest) -> WEResult<Workflow> {
        let workflow_id = sqlx::query_scalar("select workflow.create_workflow($1,$2)")
            .bind(request.name)
            .bind(request.tasks)
            .fetch_one(&self.pool)
            .await?;
        match self.read_one(&workflow_id).await {
            Ok(Some(workflow)) => Ok(workflow),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    pub async fn read_one(&self, workflow_id: &WorkflowId) -> WEResult<Option<Workflow>> {
        let result = sqlx::query_as(
            r#"
            select w.workflow_id, w.name, w.tasks
            from workflow.v_workflows w
            where w.workflow_id = $1"#,
        )
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> WEResult<Vec<Workflow>> {
        let result = sqlx::query_as(
            r#"
            select w.workflow_id, w.name, w.tasks
            from workflow.v_workflows w"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn deprecate(&self, request: WorkflowDeprecationRequest) -> WEResult<i64> {
        sqlx::query("call workflow.deprecate_workflow($1,$2)")
            .bind(request.workflow_id)
            .bind(request.new_workflow_id)
            .execute(&self.pool)
            .await?;
        Ok(request.workflow_id)
    }
}
