use common::{
    api::ApiRequestValidator,
    database::postgres::Postgres,
    error::{EmError, EmResult},
};
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    PgPool,
};

use crate::workflow::{
    data::{
        Task, TaskId, TaskRequest, TaskRequestValidator, Workflow, WorkflowDeprecationRequest,
        WorkflowId, WorkflowRequest, WorkflowRequestValidator, WorkflowTask, WorkflowTaskRequest,
    },
    service::{TaskService, WorkflowsService},
};

impl PgHasArrayType for WorkflowTask {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_task")
    }
}

impl PgHasArrayType for WorkflowTaskRequest {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_task_request")
    }
}

/// Postgres implementation of [WorkflowsService]
#[derive(Clone)]
pub struct PgWorkflowsService {
    pool: PgPool,
}

impl WorkflowsService for PgWorkflowsService {
    type Database = Postgres;
    type RequestValidator = WorkflowRequestValidator;

    fn create(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create_workflow(&self, request: &WorkflowRequest) -> EmResult<Workflow> {
        Self::RequestValidator::validate(request)?;
        let workflow_id = sqlx::query_scalar("select workflow.create_workflow($1,$2)")
            .bind(&request.name)
            .bind(&request.tasks)
            .fetch_one(&self.pool)
            .await?;
        match self.read_one(&workflow_id).await {
            Ok(Some(workflow)) => Ok(workflow),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn read_one(&self, workflow_id: &WorkflowId) -> EmResult<Option<Workflow>> {
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

    async fn read_many(&self) -> EmResult<Vec<Workflow>> {
        let result = sqlx::query_as(
            r#"
            select w.workflow_id, w.name, w.tasks
            from workflow.v_workflows w"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn deprecate(&self, request: &WorkflowDeprecationRequest) -> EmResult<WorkflowId> {
        sqlx::query("call workflow.deprecate_workflow($1,$2)")
            .bind(request.workflow_id)
            .bind(request.new_workflow_id)
            .execute(&self.pool)
            .await?;
        Ok(request.workflow_id)
    }
}

/// Postgres implementation of [TaskService]
#[derive(Clone)]
pub struct PgTasksService {
    pool: PgPool,
}

impl TaskService for PgTasksService {
    type Database = Postgres;
    type RequestValidator = TaskRequestValidator;

    fn create(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create_task(&self, request: &TaskRequest) -> EmResult<Task> {
        let task_id: TaskId = sqlx::query_scalar("select workflow.create_task($1,$2,$3,$4)")
            .bind(&request.name)
            .bind(&request.description)
            .bind(request.task_service_id)
            .bind(&request.url)
            .fetch_one(&self.pool)
            .await?;
        self.read_one(&task_id).await
    }

    async fn read_one(&self, task_id: &TaskId) -> EmResult<Task> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from workflow.v_tasks
            where task_id = $1"#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;
        result.map_or_else(
            || {
                Err(EmError::MissingRecord {
                    pk: task_id.to_string(),
                })
            },
            Ok,
        )
    }

    async fn read_many(&self) -> EmResult<Vec<Task>> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from workflow.v_tasks"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, task_id: &TaskId, request: &TaskRequest) -> EmResult<Task> {
        sqlx::query("call workflow.update_task($1,$2,$3,$4,$5)")
            .bind(task_id)
            .bind(&request.name)
            .bind(&request.description)
            .bind(request.task_service_id)
            .bind(&request.url)
            .execute(&self.pool)
            .await?;
        self.read_one(task_id).await
    }
}
