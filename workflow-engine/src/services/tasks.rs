use common::{api::ApiRequestValidator, database::Database, error::EmResult};
use serde::{Deserialize, Serialize};

/// Task data type representing a row from `task.v_tasks`
#[derive(sqlx::FromRow, Serialize)]
pub struct Task {
    /// ID of the task
    pub(crate) task_id: TaskId,
    /// Name of the task
    pub(crate) name: String,
    /// Short description of the task
    pub(crate) description: String,
    /// Url to be called as per the task execution
    pub(crate) url: String,
    /// Name of the task service that executes this task
    pub(crate) task_service_name: String,
}

/// Data required to create or update the contents of task entry (the id cannot be updated)
#[derive(Deserialize, Debug)]
pub struct TaskRequest {
    /// Name of the task
    pub(crate) name: String,
    /// Short description of the task
    pub(crate) description: String,
    /// ID of the service that executes this task
    pub(crate) task_service_id: i64,
    /// Relative url from the task service referenced by `task_service_id`
    pub(crate) url: String,
}

pub struct TaskRequestValidator;

impl ApiRequestValidator for TaskRequestValidator {
    type ErrorMessage = &'static str;
    type Request = TaskRequest;

    fn validate(request: &Self::Request) -> Result<(), Self::ErrorMessage> {
        if request.name.trim().is_empty() {
            return Err("Request 'name' cannot be empty or whitespace");
        }
        if request.description.trim().is_empty() {
            return Err("Request 'description' cannot be empty or whitespace");
        }
        if request.url.trim().is_empty() {
            return Err("Request 'url' cannot be empty or whitespace");
        }
        Ok(())
    }
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

/// Service for fetching and interacting with task data. Wraps a `pool` and provides interaction
/// methods for the API.
pub trait TaskService
where
    Self: Clone + Send,
{
    type Database: Database;
    type RequestValidator: ApiRequestValidator<Request = TaskRequest>;

    /// Create a new [TaskService] with the referenced pool as the data source
    fn create(pool: &<Self::Database as Database>::ConnectionPool) -> Self;
    /// Create a new task with the data contained within `request`
    async fn create_task(&self, request: &TaskRequest) -> EmResult<Task>;
    /// Read a single task record from `task.v_tasks` for the specified `task_id`. Will return
    /// [Err] when the id does not match a record.
    async fn read_one(&self, task_id: &TaskId) -> EmResult<Task>;
    /// Read all task records found from `task.v_tasks`
    async fn read_many(&self) -> EmResult<Vec<Task>>;
    /// Update a task specified by `task_id` with the new details contained within `request`
    async fn update(&self, task_id: &TaskId, request: &TaskRequest) -> EmResult<Task>;
}
