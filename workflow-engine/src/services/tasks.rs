use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool};

/// Task data type representing a row from `task.v_tasks`
#[derive(sqlx::FromRow, Serialize)]
pub struct Task {
    pub(crate) task_id: TaskId,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) url: String,
    pub(crate) task_service_name: String,
}

/// Data required to create or update the contents of task entry (the id cannot be updated)
#[derive(Deserialize)]
pub struct TaskRequest {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) task_service_id: i64,
    pub(crate) url: String,
}

/// Wrapper for a `task_id` value. Made to ensure data passed as the id of a task is correct and
/// not just any i64 value.
#[derive(sqlx::Type, Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
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

/// Service for fetching and interacting with task data. Wraps a [Pool] and provides
/// interaction methods for the API.
pub trait TaskService: Clone + Send {
    type Database: Database;

    /// Create a new [TaskService] with the referenced pool as the data source
    fn create(pool: &Pool<Self::Database>) -> Self;
    /// Create a new task with the data contained within `request`
    async fn create_task(&self, request: &TaskRequest) -> EmResult<Task>;
    /// Read a single task record from `task.v_tasks` for the specified `task_id`. Will return
    /// [None] when the id does not match a record.
    async fn read_one(&self, task_id: &TaskId) -> EmResult<Task>;
    /// Read all task records found from `task.v_tasks`
    async fn read_many(&self) -> EmResult<Vec<Task>>;
    /// Update a task specified by `task_id` with the new details contained within `request`
    async fn update(&self, task_id: &TaskId, request: TaskRequest) -> EmResult<Task>;
}
