use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{database::finish_transaction, error::Result as WEResult};

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

pub struct TasksService {
    pool: &'static PgPool,
}

impl TasksService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: TaskRequest) -> WEResult<Task> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query_scalar("select create_task($1,$2,$3,$4)")
            .bind(request.name)
            .bind(request.description)
            .bind(request.task_service_id)
            .bind(request.url)
            .fetch_one(&mut transaction)
            .await;
        let task_id: i64 = finish_transaction(transaction, result).await?;
        match self.read_one(task_id).await {
            Ok(Some(task)) => Ok(task),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    pub async fn read_one(&self, task_id: i64) -> WEResult<Option<Task>> {
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

    pub async fn update(&self, task_id: i64, request: TaskRequest) -> WEResult<Option<Task>> {
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
