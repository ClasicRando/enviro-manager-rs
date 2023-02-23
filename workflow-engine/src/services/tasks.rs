use rocket::request::FromParam;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    database::finish_transaction,
    error::{Error as WEError, Result as WEResult},
};

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

#[derive(sqlx::FromRow, Serialize)]
pub struct Task {
    task_id: i64,
    name: String,
    description: String,
    url: String,
    task_service_name: String,
}

#[derive(Deserialize)]
pub struct TaskRequest {
    name: String,
    description: String,
    task_service_id: i64,
    url: String,
}

#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct TaskId(i64);

impl From<i64> for TaskId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl<'a> FromParam<'a> for TaskId {
    type Error = WEError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(param.parse::<i64>()?))
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct TasksService {
    pool: &'static PgPool,
}

impl TasksService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: TaskRequest) -> WEResult<Task> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from create_task($1,$2,$3,$4)"#,
        )
        .bind(request.name)
        .bind(request.description)
        .bind(request.task_service_id)
        .bind(request.url)
        .fetch_one(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_one(&self, task_id: &TaskId) -> WEResult<Option<Task>> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from   v_tasks
            where  task_id = $1"#,
        )
        .bind(task_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> WEResult<Vec<Task>> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from   v_tasks"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn update(&self, task_id: &TaskId, request: TaskRequest) -> WEResult<Option<Task>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call update_task($1,$2,$3,$4,$5)")
            .bind(task_id)
            .bind(request.name)
            .bind(request.description)
            .bind(request.task_service_id)
            .bind(request.url)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(task_id).await
    }
}
