use rocket::request::FromParam;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    PgPool,
};

use crate::error::{Error as WEError, Result as WEResult};

/// Task data as it can be seen from it's parent, a [Workflow] instance. Contains the underlining
/// data you would find in `task.workflow_tasks` as well as task details from `task.tasks` and the
/// task service information fetched from `task.task_services`. Backed by a composite type in the
/// Postgresql database.
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

/// Task information required to create a `task.workflow_tasks` entry. 1 or more entries can be
/// found within the [WorkflowRequest] type used by the API.
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

/// API request body when attempting to create a new `workflow.workflows` entry. Defines the name
/// and tasks found within the workflow.
#[derive(Deserialize)]
pub struct WorkflowRequest {
    name: String,
    tasks: Vec<WorkflowTaskRequest>,
}

/// API request body when attempting to deprecate an existing `workflow.workflows` record. Specifies
/// the `workflow_id` as well as an optional `new_workflow_id` that replaces the old workflow.
#[derive(Deserialize)]
pub struct WorkflowDeprecationRequest {
    workflow_id: i64,
    new_workflow_id: Option<i64>,
}

/// Query result from the `workflow.v_workflows` view. Represents a workflow entry with all the
/// tasks packed into an array.
#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Workflow {
    workflow_id: i64,
    name: String,
    tasks: Vec<WorkflowTask>,
}

/// Wrapper for a `workflow_id` value. Made to ensure data passed as the id of a workflow is correct
/// and not just any i64 value.
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

/// Service for fetching and interacting with workflow run data. Wraps a [PgPool] and provides
/// interaction methods for the API.
#[derive(Clone)]
pub struct WorkflowsService {
    pool: PgPool,
}

impl WorkflowsService {
    /// Create a new [WorkflowsService] with the referenced pool as the data source
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    /// Create a new workflow using the `request` data to call the `workflow.create_workflow`
    /// procedure. Returns the new [Workflow] created.
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

    /// Read a single [Workflow] record for the specified `workflow_id`. Returns [None] if the id
    /// does not match any record in the database.
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

    /// Read all [Workflow] records in the database
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

    /// Deprecate the workflow specified within the `request` data, pointing to a new workflow
    /// if the `request` contains a `new_workflow_id` value. Returns the `workflow_id` that was
    /// updated as a response.
    pub async fn deprecate(&self, request: WorkflowDeprecationRequest) -> WEResult<i64> {
        sqlx::query("call workflow.deprecate_workflow($1,$2)")
            .bind(request.workflow_id)
            .bind(request.new_workflow_id)
            .execute(&self.pool)
            .await?;
        Ok(request.workflow_id)
    }
}
