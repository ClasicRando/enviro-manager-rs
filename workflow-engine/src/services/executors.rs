use chrono::NaiveDateTime;
use common::error::{EmError, EmResult};
use log::error;
use rocket::request::FromParam;
use serde::{Serialize, Deserialize};
use sqlx::{postgres::PgListener, types::ipnetwork::IpNetwork, Database, PgPool, Pool, Postgres};

use super::workflow_runs::WorkflowRunId;
use crate::{
    database::listener::{ChangeListener, PgChangeListener},
    executor::utilities::ExecutorStatusUpdate,
};

/// Status of an [Executor][crate::executor::Executor] as found in the database as a simple
/// Postgresql enum type
#[derive(sqlx::Type, Serialize, PartialEq, Debug)]
#[sqlx(type_name = "executor_status")]
pub enum ExecutorStatus {
    Active,
    Canceled,
    Shutdown,
}

/// Method of serializing an [IpNetwork] Postgresql type
fn serialize_ipnetwork<S>(addr: &IpNetwork, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.collect_str(addr)
}

/// Executor data type representing a row from `executor.v_executor`
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

/// Wrapper for an `executor_id` value. Made to ensure data passed as the id of an executor is
/// correct and not just any i64 value.
#[derive(sqlx::Type, Clone, Deserialize)]
#[sqlx(transparent)]
pub struct ExecutorId(i64);

impl<'a> FromParam<'a> for ExecutorId {
    type Error = EmError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(param.parse::<i64>()?))
    }
}

impl std::fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Service for fetching and interacting with executor data. Wraps a [Pool] and provides
/// interaction methods for the API and [Executor][crate::executor::Executor] instances. To
/// implement the trait you must specify the [Database] you are working with and the
/// [ChangeListener] the service will provide.
pub trait ExecutorsService : Clone {
    type Database: Database;
    type Listener: ChangeListener;

    fn new(pool: &Pool<Self::Database>) -> Self;
    /// Register a new executor with the database. Creates a record for future processes to
    /// attribute workflow runs to the new executor.
    async fn register_executor(&self) -> EmResult<ExecutorId>;
    /// Read the [Executor] record to gain information about the specified `executor_id`. If no
    /// executor matches the id provided, [None] will be returned.
    async fn read_one(&self, executor_id: &ExecutorId) -> EmResult<Option<Executor>>;
    /// Read the [ExecutorStatus] for the specified `executor_id`. If no executor matches the id
    /// provided, [None] will be returned.
    async fn read_status(&self, executor_id: &ExecutorId) -> EmResult<Option<ExecutorStatus>>;
    /// Read all [Executor] records, including instances that are inactive or marked as active but
    /// the underling session/pool is no longer active.
    async fn read_many(&self) -> EmResult<Vec<Executor>>;
    /// Read all [Executor] records, excluding those that are labeled as inactive. The output does
    /// include records with an underlining session/pool that is no longer active.
    async fn read_active(&self) -> EmResult<Vec<Executor>>;
    /// Process the next workflow run, setting it's state for execution before returning the
    /// [WorkflowRunId]. If no workflow run is available, then the function returns [None].
    async fn next_workflow_run(&self, executor_id: &ExecutorId) -> EmResult<Option<WorkflowRunId>>;
    /// Update the status of the executor specified by `executor_id` to [ExecutorStatus::Shutdown].
    /// This internally sends a signal to the [Executor][crate::executor::Executor] instance to
    /// gracefully shutdown all operation and close.
    async fn shutdown(&self, executor_id: &ExecutorId) -> EmResult<Option<Executor>>;
    /// Update the status of the executor specified by `executor_id` to [ExecutorStatus::Canceled].
    /// This internally sends a signal to the [Executor][crate::executor::Executor] instance to
    /// forcefully shutdown all operation and close.
    async fn cancel(&self, executor_id: &ExecutorId) -> EmResult<Option<Executor>>;
    /// Clean up database entries linked to the `executor_id` specified. Acts as the final step to
    /// ending an [Executor][crate::executor::Executor] instance and should only be called from
    /// the [Executor][crate::executor::Executor] itself.
    async fn close(&self, executor_id: &ExecutorId, is_cancelled: bool) -> EmResult<()>;
    /// Post the specified `error` message to the `executor_id` record. If the SQL call happens to
    /// fail that error will be logged alongside the original `error`.
    async fn post_error(&self, executor_id: &ExecutorId, error: EmError) -> EmResult<()>;
    /// Clean executor database records, setting correct statuses for executors that are no longer
    /// alive but marked as active.
    async fn clean_executors(&self) -> EmResult<()>;
    /// Get a new [ChangeListener] for the executor status update channel. Channel name is specific
    /// to the executor's id.
    async fn status_listener(&self, executor_id: &ExecutorId) -> EmResult<Self::Listener>;
}

/// Postgresql implementation of the [ExecutorsService]. Wraps a [PgPool] and provides interaction
/// methods for the API and [Executor][crate::executor::Executor] instances.
#[derive(Clone)]
pub struct PgExecutorsService {
    pool: PgPool,
}

impl ExecutorsService for PgExecutorsService {
    type Database = Postgres;
    type Listener = PgChangeListener<ExecutorStatusUpdate>;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn register_executor(&self) -> EmResult<ExecutorId> {
        let executor_id = sqlx::query_scalar("select executor.register_executor()")
            .fetch_one(&self.pool)
            .await?;
        Ok(executor_id)
    }

    async fn read_one(&self, executor_id: &ExecutorId) -> EmResult<Option<Executor>> {
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
        Ok(result)
    }

    async fn read_status(&self, executor_id: &ExecutorId) -> EmResult<Option<ExecutorStatus>> {
        let result = sqlx::query_scalar(
            r#"
            select e.status
            from executor.executors e
            where e.executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
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
            from executor.v_active_executors e"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn next_workflow_run(&self, executor_id: &ExecutorId) -> EmResult<Option<WorkflowRunId>> {
        let workflow_run_id: Option<WorkflowRunId> =
            sqlx::query_scalar("call workflow.process_next_workflow($1,$2)")
                .bind(executor_id)
                .bind(None::<i64>)
                .fetch_one(&self.pool)
                .await?;
        Ok(workflow_run_id)
    }

    async fn shutdown(&self, executor_id: &ExecutorId) -> EmResult<Option<Executor>> {
        sqlx::query("call executor.shutdown_executor($1)")
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    async fn cancel(&self, executor_id: &ExecutorId) -> EmResult<Option<Executor>> {
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

    async fn post_error(&self, executor_id: &ExecutorId, error: EmError) -> EmResult<()> {
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
        Ok(())
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

#[cfg(test)]
mod test {
    use common::error::EmError;
    use indoc::indoc;

    use super::{ExecutorStatus, ExecutorsService, PgExecutorsService};
    use crate::{
        database::{listener::ChangeListener, utilities::create_test_db_pool},
        executor::utilities::ExecutorStatusUpdate,
    };

    #[sqlx::test]
    async fn create_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = PgExecutorsService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        if executor_service.read_one(&executor_id).await?.is_none() {
            panic!("Failed to `read_one`");
        };

        let Some(executor_status) = executor_service.read_status(&executor_id).await? else {
            panic!("Failed to `read_status`");
        };
        assert_eq!(executor_status, ExecutorStatus::Active);
        Ok(())
    }

    #[sqlx::test]
    async fn cancel_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = PgExecutorsService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        if executor_service.cancel(&executor_id).await?.is_none() {
            panic!("Failed to `cancel`");
        };
        let Some(executor_status) = executor_service.read_status(&executor_id).await? else {
            panic!("Failed to `read_status`");
        };
        assert_eq!(executor_status, ExecutorStatus::Canceled);

        Ok(())
    }

    #[sqlx::test]
    async fn shutdown_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = PgExecutorsService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        if executor_service.shutdown(&executor_id).await?.is_none() {
            panic!("Failed to `shutdown`");
        };
        let Some(executor_status) = executor_service.read_status(&executor_id).await? else {
            panic!("Failed to `read_status`");
        };
        assert_eq!(executor_status, ExecutorStatus::Shutdown);

        Ok(())
    }

    #[sqlx::test]
    async fn post_error() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = PgExecutorsService::new(&pool);

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let error = EmError::Generic(String::from("Executor 'post_error' test"));
        let error_message = error.to_string();
        executor_service.post_error(&executor_id, error).await?;

        let query = indoc! {
            r#"
            select e.error_message
            from executor.executors e
            where e.executor_id = $1"#
        };
        let message: String = sqlx::query_scalar(query)
            .bind(&executor_id)
            .fetch_one(&pool)
            .await?;
        assert_eq!(message, error_message);

        Ok(())
    }

    #[sqlx::test]
    async fn status_listener() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = PgExecutorsService::new(&pool);

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let mut listener = executor_service.status_listener(&executor_id).await?;

        sqlx::query(&format!("NOTIFY exec_status_{}, 'Test'", executor_id,))
            .execute(&pool)
            .await?;
        let update = listener.recv().await?;

        assert_eq!(ExecutorStatusUpdate::NoOp, update);

        Ok(())
    }

    #[sqlx::test]
    async fn clean_executors() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = PgExecutorsService::new(&pool);

        let inactive_executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        drop(executor_service);
        pool.close().await;
        drop(pool);

        let pool = create_test_db_pool().await?;
        let executor_service = PgExecutorsService::new(&pool);
        executor_service.clean_executors().await?;

        let Some(status) = executor_service.read_status(&inactive_executor_id).await? else {
            panic!("Could not `read_status`");
        };

        assert_eq!(
            status,
            ExecutorStatus::Canceled,
            "Status was not Canceled for executor_id = {}",
            inactive_executor_id
        );

        Ok(())
    }
}
