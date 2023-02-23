use chrono::NaiveDateTime;
use log::error;
use rocket::request::FromParam;
use serde::Serialize;
use sqlx::{postgres::PgListener, types::ipnetwork::IpNetwork, PgPool};

use crate::error::{Error as WEError, Result as WEResult};

use super::workflow_runs::WorkflowRunId;

#[derive(sqlx::Type, Serialize)]
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

#[derive(sqlx::Type, Clone)]
#[sqlx(transparent)]
pub struct ExecutorId(i64);

impl<'a> FromParam<'a> for ExecutorId {
    type Error = WEError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(param.parse::<i64>()?))
    }
}

impl std::fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct ExecutorsService {
    pool: &'static PgPool,
}

impl ExecutorsService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn register_executor(&self) -> WEResult<ExecutorId> {
        let executor_id = sqlx::query_scalar("select register_we_executor()")
            .fetch_one(self.pool)
            .await?;
        Ok(executor_id)
    }

    pub async fn read_one(&self, executor_id: &ExecutorId) -> WEResult<Option<Executor>> {
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

    pub async fn read_status(&self, executor_id: &ExecutorId) -> WEResult<Option<ExecutorStatus>> {
        let result = sqlx::query_scalar(
            r#"
            select status
            from   registered_we_executors
            where  executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> WEResult<Vec<Executor>> {
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

    pub async fn read_active(&self) -> WEResult<Vec<Executor>> {
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

    pub async fn next_workflow_run(
        &self,
        executor_id: &ExecutorId,
    ) -> WEResult<Option<WorkflowRunId>> {
        let workflow_run_id: Option<WorkflowRunId> =
            sqlx::query_scalar("call workflow_engine.process_next_workflow($1,$2)")
                .bind(executor_id)
                .bind(None::<i64>)
                .fetch_optional(self.pool)
                .await?;
        Ok(workflow_run_id)
    }

    pub async fn shutdown(&self, executor_id: &ExecutorId) -> WEResult<Option<Executor>> {
        sqlx::query("call shutdown_executor($1)")
            .bind(executor_id)
            .execute(self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    pub async fn cancel(&self, executor_id: &ExecutorId) -> WEResult<Option<Executor>> {
        sqlx::query("call cancel_executor($1)")
            .bind(executor_id)
            .execute(self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    pub async fn close(&self, executor_id: &ExecutorId, is_cancelled: bool) -> WEResult<()> {
        sqlx::query("call close_we_executor($1,$2)")
            .bind(executor_id)
            .bind(is_cancelled)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn post_error(&self, executor_id: &ExecutorId, error: WEError) -> WEResult<()> {
        let message = format!("{}", error);
        let sql_result = sqlx::query("call post_executor_error_message($1,$2)")
            .bind(executor_id)
            .bind(&message)
            .execute(self.pool)
            .await;
        if let Err(error) = sql_result {
            error!(
                "Error while attempting to post executor error.\n{:?}",
                error
            );
        }
        error!("Executor fatal error. {}", message);
        Ok(())
    }

    pub async fn clean_executors(&self) -> WEResult<()> {
        sqlx::query("call clean_executors()")
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn status_listener(&self, executor_id: &ExecutorId) -> WEResult<PgListener> {
        let mut listener = PgListener::connect_with(self.pool).await?;
        listener
            .listen(&format!("exec_status_{}", executor_id))
            .await?;
        Ok(listener)
    }
}
