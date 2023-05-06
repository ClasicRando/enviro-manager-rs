use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Database, PgPool, Pool, Postgres};

/// Status of a task as found in the database as a simple Postgresql enum type
#[derive(sqlx::Type, Serialize)]
#[sqlx(type_name = "task_status")]
pub enum TaskStatus {
    Waiting,
    Running,
    Complete,
    Failed,
    #[sqlx(rename = "Rule Broken")]
    RuleBroken,
    Paused,
    Canceled,
}

/// Task data type representing a row from `task.v_tasks`
#[derive(sqlx::FromRow, Serialize)]
pub struct Task {
    task_id: i64,
    name: String,
    description: String,
    url: String,
    task_service_name: String,
}

/// Data required to create or update the contents of task entry (the id cannot be updated)
#[derive(Deserialize)]
pub struct TaskRequest {
    name: String,
    description: String,
    task_service_id: i64,
    url: String,
}

/// Wrapper for a `task_id` value. Made to ensure data passed as the id of a task is correct and
/// not just any i64 value.
#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct TaskId(i64);

impl From<i64> for TaskId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait::async_trait]
pub trait TasksService: Clone + Send {
    type Database: Database;

    /// Create a new [TasksService] with the referenced pool as the data source
    fn new(pool: &Pool<Self::Database>) -> Self;
    /// Create a new task with the data contained within `request`
    async fn create(&self, request: &TaskRequest) -> EmResult<Task>;
    /// Read a single task record from `task.v_tasks` for the specified `task_id`. Will return
    /// [None] when the id does not match a record.
    async fn read_one(&self, task_id: &TaskId) -> EmResult<Option<Task>>;
    /// Read all task records found from `task.v_tasks`
    async fn read_many(&self) -> EmResult<Vec<Task>>;
    /// Update a task specified by `task_id` with the new details contained within `request`
    async fn update(&self, task_id: &TaskId, request: TaskRequest) -> EmResult<Option<Task>>;
}

/// Service for fetching and interacting with task data. Wraps a [PgPool] and provides
/// interaction methods for the API.
#[derive(Clone)]
pub struct PgTasksService {
    pool: PgPool,
}

#[async_trait::async_trait]
impl TasksService for PgTasksService {
    type Database = Postgres;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create(&self, request: &TaskRequest) -> EmResult<Task> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from task.create_task($1,$2,$3,$4)"#,
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.task_service_id)
        .bind(&request.url)
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    async fn read_one(&self, task_id: &TaskId) -> EmResult<Option<Task>> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from task.v_tasks
            where task_id = $1"#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    async fn read_many(&self) -> EmResult<Vec<Task>> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from task.v_tasks"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, task_id: &TaskId, request: TaskRequest) -> EmResult<Option<Task>> {
        sqlx::query("call task.update_task($1,$2,$3,$4,$5)")
            .bind(task_id)
            .bind(request.name)
            .bind(request.description)
            .bind(request.task_service_id)
            .bind(request.url)
            .execute(&self.pool)
            .await?;
        self.read_one(task_id).await
    }
}

#[cfg(test)]
mod test {
    use super::{PgTasksService, TaskId, TaskRequest, TasksService};
    use crate::database::{ConnectionPool, PostgresConnectionPool};

    #[sqlx::test]
    async fn task() -> Result<(), Box<dyn std::error::Error>> {
        let task_name = "Create Task Test";
        let task_description = "Test task created as integration testing.";
        let task_service_id = 1_i64;
        let task_url = r"test2";

        let request = TaskRequest {
            name: task_name.to_string(),
            description: task_description.to_string(),
            task_service_id,
            url: task_url.to_string(),
        };

        let pool = PostgresConnectionPool::create_test_db_pool().await?;
        let (task_service_name, service_url): (String, String) =
            sqlx::query_as("select name, base_url from task.task_services where service_id = $1")
                .bind(task_service_id)
                .fetch_one(&pool)
                .await?;
        let task_url_full = format!("{}\\{}", service_url, task_url);
        sqlx::query("delete from task.tasks where name = $1")
            .bind(task_name)
            .execute(&pool)
            .await?;

        let tasks_service = PgTasksService::new(&pool);

        let task = tasks_service.create(&request).await?;
        let task_id = TaskId::from(task.task_id);

        assert_eq!(task.name, task_name);
        assert_eq!(task.description, task_description);
        assert_eq!(task.task_service_name, task_service_name);
        assert_eq!(task.url, task_url_full);

        let Some(task) = tasks_service.read_one(&task_id).await? else {
            panic!("Failed `read_one` test");
        };

        assert_eq!(task.name, task_name);
        assert_eq!(task.description, task_description);
        assert_eq!(task.task_service_name, task_service_name);
        assert_eq!(task.url, task_url_full);

        let count: i64 = sqlx::query_scalar("select count(0) from task.tasks")
            .fetch_one(&pool)
            .await?;
        let tasks = tasks_service.read_many().await?;

        assert_eq!(count as usize, tasks.len());

        let new_description = "New Task Description";
        let request = TaskRequest {
            description: new_description.to_string(),
            ..request
        };
        let Some(task) = tasks_service.update(&task_id, request).await? else {
            panic!("Failed `update` test");
        };

        assert_eq!(task.name, task_name);
        assert_eq!(task.description, new_description);
        assert_eq!(task.task_service_name, task_service_name);
        assert_eq!(task.url, task_url_full);

        Ok(())
    }
}
