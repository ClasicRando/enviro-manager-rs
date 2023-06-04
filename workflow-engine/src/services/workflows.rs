use common::{api::ApiRequestValidator, database::Database, error::EmResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::tasks::TaskId;

/// Task data as it can be seen from it's parent, a [Workflow] instance. Contains the underlining
/// data you would find in `task.workflow_tasks` as well as task details from `task.tasks` and the
/// task service information fetched from `task.task_services`. Backed by a composite type in the
/// Postgresql database.
#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "workflow_task")]
pub struct WorkflowTask {
    /// Order of a task within a workflow run
    pub(crate) task_order: i32,
    /// ID of the task to be executed
    pub(crate) task_id: TaskId,
    /// Name of the task
    pub(crate) name: String,
    /// Short description of the task
    pub(crate) description: String,
    /// Optional parameters passed to the task executor to allow for custom behaviour
    pub(crate) parameters: Option<Value>,
    /// Name of the task service that executes this task
    pub(crate) service_name: String,
    /// Url to be called as per the task execution
    pub(crate) url: String,
}

/// Task information required to create a `task.workflow_tasks` entry. 1 or more entries can be
/// found within the [WorkflowRequest] type used by the API.
#[derive(sqlx::Type, Deserialize, Debug)]
#[sqlx(type_name = "workflow_task_request")]
pub struct WorkflowTaskRequest {
    /// ID of the task to be executed
    pub(crate) task_id: TaskId,
    /// Optional parameters passed to the task executor to allow for custom behaviour
    pub(crate) parameters: Option<Value>,
}

/// API request body when attempting to create a new `workflow.workflows` entry. Defines the name
/// and tasks found within the workflow.
#[derive(Deserialize, Debug)]
pub struct WorkflowRequest {
    /// Name of the workflow to create
    pub(crate) name: String,
    /// Tasks that are run as part of this new workflow
    pub(crate) tasks: Vec<WorkflowTaskRequest>,
}

pub struct WorkflowRequestValidator;

impl ApiRequestValidator for WorkflowRequestValidator {
    type ErrorMessage = &'static str;
    type Request = WorkflowRequest;

    fn validate(request: &Self::Request) -> Result<(), Self::ErrorMessage> {
        if request.name.trim().is_empty() {
            return Err("Request 'name' cannot be empty or whitespace");
        }
        Ok(())
    }
}

/// API request body when attempting to deprecate an existing `workflow.workflows` record. Specifies
/// the `workflow_id` as well as an optional `new_workflow_id` that replaces the old workflow.
#[derive(Deserialize, Debug)]
pub struct WorkflowDeprecationRequest {
    /// ID of the workflow to deprecate
    pub(crate) workflow_id: WorkflowId,
    /// Optional ID of the workflow to replace this deprecated workflow
    pub(crate) new_workflow_id: Option<WorkflowId>,
}

/// Query result from the `workflow.v_workflows` view. Represents a workflow entry with all the
/// tasks packed into an array.
#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct Workflow {
    /// ID of the workflow
    pub(crate) workflow_id: WorkflowId,
    /// Name of the workflow
    pub(crate) name: String,
    /// Tasks that are executed as part of this workflow
    pub(crate) tasks: Vec<WorkflowTask>,
}

/// Wrapper for a `workflow_id` value. Made to ensure data passed as the id of a workflow is correct
/// and not just any i64 value.
#[derive(sqlx::Type, Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
#[sqlx(transparent)]
pub struct WorkflowId(i64);

impl From<i64> for WorkflowId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Service for fetching and interacting with workflow run data. Wraps a [Pool] and provides
/// interaction methods for the API.
pub trait WorkflowsService
where
    Self: Clone + Send,
{
    type Database: Database;
    type RequestValidator: ApiRequestValidator<Request = WorkflowRequest>;
    /// Create a new [WorkflowsService] with the referenced pool as the data source
    fn create(pool: &<Self::Database as Database>::ConnectionPool) -> Self;
    /// Create a new workflow using the `request` data to call the `workflow.create_workflow`
    /// procedure. Returns the new [Workflow] created.
    async fn create_workflow(&self, request: &WorkflowRequest) -> EmResult<Workflow>;
    /// Read a single [Workflow] record for the specified `workflow_id`. Returns [Err] if the id
    /// does not match any record in the database.
    async fn read_one(&self, workflow_id: &WorkflowId) -> EmResult<Option<Workflow>>;
    /// Read all [Workflow] records in the database
    async fn read_many(&self) -> EmResult<Vec<Workflow>>;
    /// Deprecate the workflow specified within the `request` data, pointing to a new workflow
    /// if the `request` contains a `new_workflow_id` value. Returns the `workflow_id` that was
    /// updated as a response.
    async fn deprecate(&self, request: &WorkflowDeprecationRequest) -> EmResult<WorkflowId>;
}
