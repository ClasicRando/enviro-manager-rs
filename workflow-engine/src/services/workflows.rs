use common::error::EmResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    Database, PgPool, Pool, Postgres,
};

/// Task data as it can be seen from it's parent, a [Workflow] instance. Contains the underlining
/// data you would find in `task.workflow_tasks` as well as task details from `task.tasks` and the
/// task service information fetched from `task.task_services`. Backed by a composite type in the
/// Postgresql database.
#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
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
#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct Workflow {
    workflow_id: i64,
    name: String,
    tasks: Vec<WorkflowTask>,
}

/// Wrapper for a `workflow_id` value. Made to ensure data passed as the id of a workflow is correct
/// and not just any i64 value.
#[derive(sqlx::Type, Deserialize)]
#[sqlx(transparent)]
pub struct WorkflowId(i64);

impl From<i64> for WorkflowId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait::async_trait]
pub trait WorkflowsService: Clone + Send {
    type Database: Database;
    /// Create a new [WorkflowsService] with the referenced pool as the data source
    fn new(pool: &Pool<Self::Database>) -> Self;
    /// Create a new workflow using the `request` data to call the `workflow.create_workflow`
    /// procedure. Returns the new [Workflow] created.
    async fn create(&self, request: WorkflowRequest) -> EmResult<Workflow>;
    /// Read a single [Workflow] record for the specified `workflow_id`. Returns [None] if the id
    /// does not match any record in the database.
    async fn read_one(&self, workflow_id: &WorkflowId) -> EmResult<Option<Workflow>>;
    /// Read all [Workflow] records in the database
    async fn read_many(&self) -> EmResult<Vec<Workflow>>;
    /// Deprecate the workflow specified within the `request` data, pointing to a new workflow
    /// if the `request` contains a `new_workflow_id` value. Returns the `workflow_id` that was
    /// updated as a response.
    async fn deprecate(&self, request: WorkflowDeprecationRequest) -> EmResult<i64>;
}

/// Service for fetching and interacting with workflow run data. Wraps a [PgPool] and provides
/// interaction methods for the API.
#[derive(Clone)]
pub struct PgWorkflowsService {
    pool: PgPool,
}

#[async_trait::async_trait]
impl WorkflowsService for PgWorkflowsService {
    type Database = Postgres;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create(&self, request: WorkflowRequest) -> EmResult<Workflow> {
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

    async fn deprecate(&self, request: WorkflowDeprecationRequest) -> EmResult<i64> {
        sqlx::query("call workflow.deprecate_workflow($1,$2)")
            .bind(request.workflow_id)
            .bind(request.new_workflow_id)
            .execute(&self.pool)
            .await?;
        Ok(request.workflow_id)
    }
}

#[cfg(test)]
mod test {
    use common::error::EmResult;
    use sqlx::PgPool;

    use super::{
        PgWorkflowsService, WorkflowDeprecationRequest, WorkflowId, WorkflowRequest,
        WorkflowTaskRequest, WorkflowsService,
    };
    use crate::{
        database::{ConnectionPool, PostgresConnectionPool},
        services::workflows::Workflow,
    };

    async fn clean_test_workflow(workflow_name: &str, pool: &PgPool) -> EmResult<()> {
        sqlx::query(
            r#"
            with workflows as (
                delete from task.workflow_tasks wt
                using workflow.workflows w
                where
                    w.name = $1
                    and wt.workflow_id = w.workflow_id
                returning wt.workflow_id
            )
            delete from workflow.workflows w1
            using (
                select w1.workflow_id
                from workflows w1
                union
                select w2.workflow_id
                from workflow.workflows w2
                where w2.name = $2
            ) w2
            where w1.workflow_id = w2.workflow_id"#,
        )
        .bind(workflow_name)
        .bind(workflow_name)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn reset_base_test_workflow(pool: &PgPool) -> EmResult<()> {
        sqlx::query(
            r#"
            update workflow.workflows
            set
                is_deprecated = false,
                new_workflow = null
            where workflow_id = 1"#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    #[sqlx::test]
    async fn create() -> EmResult<()> {
        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        let service = PgWorkflowsService::new(&pool);
        let workflow_name = "Create test";
        let task_id = 1;
        let parameters = None;

        let request = WorkflowRequest {
            name: String::from(workflow_name),
            tasks: vec![WorkflowTaskRequest {
                task_id,
                parameters,
            }],
        };

        let workflow = service.create(request).await?;

        assert_eq!(workflow.name, workflow_name);
        assert_eq!(workflow.tasks.len(), 1);
        assert_eq!(workflow.tasks[0].task_id, task_id);

        clean_test_workflow(workflow_name, &pool).await?;
        let workflow = service.read_one(&WorkflowId(workflow.workflow_id)).await?;
        assert!(workflow.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn read_one_should_return_some_when_record_exists() -> EmResult<()> {
        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        let service = PgWorkflowsService::new(&pool);
        let workflow_id = 1;
        let workflow_name = "test";
        let task_id = 1;

        let Some(workflow) = service.read_one(&WorkflowId(workflow_id)).await? else {
            panic!("Record not found")
        };

        assert_eq!(workflow.name, workflow_name);
        assert_eq!(workflow.tasks.len(), 1);
        assert_eq!(workflow.tasks[0].task_id, task_id);

        Ok(())
    }

    #[sqlx::test]
    async fn read_one_should_return_none_when_record_exists() -> EmResult<()> {
        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        let service = PgWorkflowsService::new(&pool);
        let workflow_id = -1;

        let result = service.read_one(&WorkflowId(workflow_id)).await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn read_many() -> EmResult<()> {
        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        let service = PgWorkflowsService::new(&pool);

        let workflows = service.read_many().await?;

        assert!(!workflows.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn deprecate_workflow_with_no_new_workflow() -> EmResult<()> {
        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        let service = PgWorkflowsService::new(&pool);

        let workflow_name = "deprecate no new workflow test";
        let task_id = 1;
        let parameters = None;

        let request = WorkflowRequest {
            name: String::from(workflow_name),
            tasks: vec![WorkflowTaskRequest {
                task_id,
                parameters,
            }],
        };

        let Workflow {
            workflow_id: created_workflow_id,
            ..
        } = service.create(request).await?;

        let request = WorkflowDeprecationRequest {
            workflow_id: created_workflow_id,
            new_workflow_id: None,
        };

        let return_workflow_id = service.deprecate(request).await?;
        let (is_deprecated, new_workflow_id): (bool, Option<i64>) = sqlx::query_as(
            r#"
            select w.is_deprecated, w.new_workflow
            from workflow.workflows w
            where w.workflow_id = $1"#,
        )
        .bind(created_workflow_id)
        .fetch_one(&pool)
        .await?;

        assert_eq!(created_workflow_id, return_workflow_id);
        assert_eq!(new_workflow_id, None);
        assert!(is_deprecated);

        reset_base_test_workflow(&pool).await?;

        Ok(())
    }

    #[sqlx::test]
    async fn deprecate_workflow_with_new_workflow() -> EmResult<()> {
        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        let service = PgWorkflowsService::new(&pool);
        let workflow_id = 1;
        let new_workflow_name = "deprecate workflow new workflow test";
        let task_id = 1;
        let parameters = None;

        let request = WorkflowRequest {
            name: String::from(new_workflow_name),
            tasks: vec![WorkflowTaskRequest {
                task_id,
                parameters,
            }],
        };

        let Workflow {
            workflow_id: created_workflow_id,
            ..
        } = service.create(request).await?;

        let request = WorkflowDeprecationRequest {
            workflow_id,
            new_workflow_id: Some(created_workflow_id),
        };

        let return_workflow_id = service.deprecate(request).await?;
        let (is_deprecated, new_workflow_id): (bool, Option<i64>) = sqlx::query_as(
            r#"
            select w.is_deprecated, w.new_workflow
            from workflow.workflows w
            where w.workflow_id = $1"#,
        )
        .bind(workflow_id)
        .fetch_one(&pool)
        .await?;

        assert_eq!(workflow_id, return_workflow_id);
        assert_eq!(new_workflow_id, Some(created_workflow_id));
        assert!(is_deprecated);

        clean_test_workflow(new_workflow_name, &pool).await?;
        let workflow = service.read_one(&WorkflowId(created_workflow_id)).await?;
        assert!(workflow.is_none());

        reset_base_test_workflow(&pool).await?;

        Ok(())
    }
}
