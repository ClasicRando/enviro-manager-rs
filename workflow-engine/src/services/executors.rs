use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{types::ipnetwork::IpNetwork, PgPool};

use crate::database::finish_transaction;

use super::error::ServiceResult;

#[derive(sqlx::Type)]
#[sqlx(type_name = "executor_status")]
pub enum ExecutorStatus {
    Active,
    Canceled,
    Shutdown,
}

fn serialize_ipnetwork<S>(addr: &IpNetwork, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.collect_str(addr)
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Executor {
    executor_id: i64,
    pid: i32,
    username: String,
    application_name: String,
    #[serde(serialize_with = "serialize_ipnetwork")]
    client_addr: IpNetwork,
    client_port: i32,
    exec_start: NaiveDateTime,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    session_active: Option<bool>,
    #[sqlx(default, rename = "wr_count")]
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow_run_count: Option<i64>,
}

struct ExecutorService {
    pool: &'static PgPool,
}

impl ExecutorService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn read_one(&self, executor_id: i64) -> ServiceResult<Option<Executor>> {
        let result = sqlx::query_as(
            r#"
            select executor_id, pid, username, application_name, client_addr, client_port, exec_start
                   session_active, workflow_run_count
            from   v_executors
            where  executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> ServiceResult<Vec<Executor>> {
        let result = sqlx::query_as(
            r#"
            select executor_id, pid, username, application_name, client_addr, client_port, exec_start
                   session_active, workflow_run_count
            from   v_executors"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_active(&self) -> ServiceResult<Vec<Executor>> {
        let result = sqlx::query_as(
            r#"
            select executor_id, pid, username, application_name, client_addr, client_port, exec_start
                   session_active, workflow_run_count
            from   v_executors
            where  status = 'Active'::executor_status"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn shutdown(&self, executor_id: i64) -> ServiceResult<Option<Executor>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call shutdown_executor($1)")
            .bind(executor_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(executor_id).await
    }

    pub async fn cancel(&self, executor_id: i64) -> ServiceResult<Option<Executor>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call cancel_executor($1)")
            .bind(executor_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(executor_id).await
    }
}
