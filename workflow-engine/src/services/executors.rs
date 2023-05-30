use chrono::NaiveDateTime;
use common::{
    database::{listener::ChangeListener, Database},
    error::{EmError, EmResult},
};
use serde::{Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;

use super::workflow_runs::WorkflowRunId;
use crate::executor::utilities::ExecutorStatusUpdate;

/// Status of an [Executor][crate::executor::Executor] as found in the database as a simple
/// Postgresql enum type
#[derive(sqlx::Type, Serialize, PartialEq, Debug)]
#[sqlx(type_name = "executor_status")]
pub enum ExecutorStatus {
    Active,
    Canceled,
    Shutdown,
}

/// Method of serializing an [IpNetwork] type
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

impl std::fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Service for fetching and interacting with executor data. Wraps a [Pool] and provides
/// interaction methods for the API and [Executor][crate::executor::Executor] instances. To
/// implement the trait you must specify the [Database] you are working with and the
/// [ChangeListener] the service will provide.
pub trait ExecutorService
where
    Self: Clone + Send,
{
    type Database: Database;
    type Listener: ChangeListener<Message = ExecutorStatusUpdate>;

    fn create(pool: &<Self::Database as Database>::ConnectionPool) -> Self;
    /// Register a new executor with the database. Creates a record for future processes to
    /// attribute workflow runs to the new executor.
    async fn register_executor(&self) -> EmResult<ExecutorId>;
    /// Read the [Executor] record to gain information about the specified `executor_id`. If no
    /// executor matches the id provided, [None] will be returned.
    async fn read_one(&self, executor_id: &ExecutorId) -> EmResult<Executor>;
    /// Read the [ExecutorStatus] for the specified `executor_id`. If no executor matches the id
    /// provided, [Err] will be returned.
    async fn read_status(&self, executor_id: &ExecutorId) -> EmResult<ExecutorStatus>;
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
    async fn shutdown(&self, executor_id: &ExecutorId) -> EmResult<Executor>;
    /// Update the status of the executor specified by `executor_id` to [ExecutorStatus::Canceled].
    /// This internally sends a signal to the [Executor][crate::executor::Executor] instance to
    /// forcefully shutdown all operation and close.
    async fn cancel(&self, executor_id: &ExecutorId) -> EmResult<Executor>;
    /// Clean up database entries linked to the `executor_id` specified. Acts as the final step to
    /// ending an [Executor][crate::executor::Executor] instance and should only be called from
    /// the [Executor][crate::executor::Executor] itself.
    async fn close(&self, executor_id: &ExecutorId, is_cancelled: bool) -> EmResult<()>;
    /// Post the specified `error` message to the `executor_id` record. If the SQL call happens to
    /// fail that error will be logged alongside the original `error`.
    async fn post_error(&self, executor_id: &ExecutorId, error: EmError);
    /// Clean executor database records, setting correct statuses for executors that are no longer
    /// alive but marked as active.
    async fn clean_executors(&self) -> EmResult<()>;
    /// Get a new [ChangeListener] for the executor status update channel. Channel name is specific
    /// to the executor's id.
    async fn status_listener(&self, executor_id: &ExecutorId) -> EmResult<Self::Listener>;
}
