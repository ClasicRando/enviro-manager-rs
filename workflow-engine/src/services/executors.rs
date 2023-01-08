use chrono::NaiveDateTime;
use log::error;
use serde::Serialize;
use sqlx::{postgres::PgListener, types::ipnetwork::IpNetwork, PgPool, Postgres, Transaction};

use crate::{
    database::finish_transaction,
    error::{Error as WEError, Result as WEResult},
};

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

pub struct ExecutorsService {
    pool: &'static PgPool,
}

impl ExecutorsService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn register_executor(&self) -> WEResult<i64> {
        let executor_id = sqlx::query_scalar("select register_we_executor()")
            .fetch_one(self.pool)
            .await?;
        Ok(executor_id)
    }

    pub async fn read_one(&self, executor_id: i64) -> WEResult<Option<Executor>> {
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

    pub async fn read_status(&self, executor_id: i64) -> WEResult<Option<ExecutorStatus>> {
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

    async fn start_workflow_run(
        &self,
        workflow_run_id: i64,
        executor_id: i64,
        mut transaction: Transaction<'_, Postgres>,
    ) -> WEResult<Option<i64>> {
        let result = sqlx::query("call start_workflow_run($1, $2)")
            .bind(workflow_run_id)
            .bind(executor_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        Ok(Some(workflow_run_id))
    }

    async fn complete_workflow_run(
        &self,
        workflow_run_id: i64,
        mut transaction: Transaction<'_, Postgres>,
    ) -> WEResult<Option<i64>> {
        let result = sqlx::query("call complete_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        Ok(None)
    }

    async fn process_next_workflow_run(
        &self,
        executor_id: i64,
        fetch_result: Result<Option<(i64, bool)>, sqlx::Error>,
        transaction: Transaction<'_, Postgres>,
    ) -> WEResult<Option<i64>> {
        match fetch_result {
            Ok(Some((workflow_run_id, true))) => {
                self.start_workflow_run(workflow_run_id, executor_id, transaction)
                    .await
            }
            Ok(Some((workflow_run_id, false))) => {
                self.complete_workflow_run(workflow_run_id, transaction)
                    .await
            }
            Ok(None) => {
                transaction.commit().await?;
                Ok(None)
            }
            Err(error) => {
                transaction.rollback().await?;
                Err(error.into())
            }
        }
    }

    pub async fn next_workflow_run(&self, executor_id: i64) -> WEResult<Option<i64>> {
        let mut transaction = self.pool.begin().await?;
        let fetch_result = sqlx::query_as(
            r#"
            select workflow_run_id, is_valid
            from   next_workflow($1)"#,
        )
        .bind(executor_id)
        .fetch_optional(&mut transaction)
        .await;
        let workflow_run_id = self
            .process_next_workflow_run(executor_id, fetch_result, transaction)
            .await?;
        Ok(workflow_run_id)
    }

    pub async fn shutdown(&self, executor_id: i64) -> WEResult<Option<Executor>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call shutdown_executor($1)")
            .bind(executor_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(executor_id).await
    }

    pub async fn cancel(&self, executor_id: i64) -> WEResult<Option<Executor>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call cancel_executor($1)")
            .bind(executor_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(executor_id).await
    }

    pub async fn close(&self, executor_id: i64, is_cancelled: bool) -> WEResult<()> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call close_we_executor($1,$2)")
            .bind(executor_id)
            .bind(is_cancelled)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        Ok(())
    }

    pub async fn post_error(&self, executor_id: i64, error: WEError) -> WEResult<()> {
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

    pub async fn status_listener(&self, executor_id: i64) -> WEResult<PgListener> {
        let mut listener = PgListener::connect_with(self.pool).await?;
        listener
            .listen(&format!("exec_status_{}", executor_id))
            .await?;
        Ok(listener)
    }
}
