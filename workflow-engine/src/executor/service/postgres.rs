use common::{
    database::postgres::{listener::PgChangeListener, Postgres},
    error::{EmError, EmResult},
};
use log::error;
use sqlx::{postgres::PgListener, PgPool};

use crate::executor::{
    data::{Executor, ExecutorId, ExecutorStatus},
    service::ExecutorService,
    utilities::ExecutorStatusUpdate,
};

/// Postgresql implementation of the [ExecutorService]. Wraps a [PgPool] and provides interaction
/// methods for the API and [Executor][crate::executor::Executor] instances.
#[derive(Clone)]
pub struct PgExecutorService {
    pool: PgPool,
}

impl PgExecutorService {
    /// Create a new instance of [PgExecutorService] using the data source provided
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}

impl ExecutorService for PgExecutorService {
    type Database = Postgres;
    type Listener = PgChangeListener<ExecutorStatusUpdate>;

    async fn register_executor(&self) -> EmResult<ExecutorId> {
        let executor_id = sqlx::query_scalar("select executor.register_executor()")
            .fetch_one(&self.pool)
            .await?;
        Ok(executor_id)
    }

    async fn read_one(&self, executor_id: &ExecutorId) -> EmResult<Executor> {
        let result = sqlx::query_as(
            r#"
            select
                e.executor_id, e.pid, e.username, e.application_name, e.client_addr, e.client_port,
                e.exec_start, e.session_active, e.wr_count
            from executor.v_executors e
            where e.executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(&self.pool)
        .await?;
        result.map_or_else(
            || {
                Err(EmError::MissingRecord {
                    pk: executor_id.to_string(),
                })
            },
            Ok,
        )
    }

    async fn read_status(&self, executor_id: &ExecutorId) -> EmResult<ExecutorStatus> {
        let result = sqlx::query_scalar(
            r#"
            select e.status
            from executor.v_executors e
            where e.executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(&self.pool)
        .await?;
        result.map_or_else(
            || {
                Err(EmError::MissingRecord {
                    pk: executor_id.to_string(),
                })
            },
            Ok,
        )
    }

    async fn read_many(&self) -> EmResult<Vec<Executor>> {
        let result = sqlx::query_as(
            r#"
            select
                e.executor_id, e.pid, e.username, e.application_name, e.client_addr, e.client_port,
                e.exec_start, e.session_active, e.wr_count
            from executor.v_executors e"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn read_active(&self) -> EmResult<Vec<Executor>> {
        let result = sqlx::query_as(
            r#"
            select
                e.executor_id, e.pid, e.username, e.application_name, e.client_addr, e.client_port,
                e.exec_start, e.session_active, e.wr_count
            from executor.v_executors e
            where e.status = 'Active'::executor.executor_status"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn shutdown(&self, executor_id: &ExecutorId) -> EmResult<Executor> {
        sqlx::query("call executor.shutdown_executor($1)")
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    async fn cancel(&self, executor_id: &ExecutorId) -> EmResult<Executor> {
        sqlx::query("call executor.cancel_executor($1)")
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    async fn close(&self, executor_id: &ExecutorId, is_cancelled: bool) -> EmResult<()> {
        sqlx::query("call executor.close_executor($1,$2)")
            .bind(executor_id)
            .bind(is_cancelled)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn post_error(&self, executor_id: &ExecutorId, error: EmError) {
        let message = format!("{}", error);
        let sql_result = sqlx::query("call executor.post_executor_error_message($1,$2)")
            .bind(executor_id)
            .bind(&message)
            .execute(&self.pool)
            .await;
        if let Err(error) = sql_result {
            error!(
                "Error while attempting to post executor error.\n{:?}",
                error
            );
        }
        error!("Executor fatal error. {}", message);
    }

    async fn clean_executors(&self) -> EmResult<()> {
        sqlx::query("call executor.clean_executors()")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn status_listener(&self, executor_id: &ExecutorId) -> EmResult<Self::Listener> {
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener
            .listen(&format!("exec_status_{}", executor_id))
            .await?;
        Ok(PgChangeListener::new(listener))
    }
}
