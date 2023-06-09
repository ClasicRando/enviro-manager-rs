pub mod postgres;

use common::{api::ApiRequestValidator, database::Database, error::EmResult};

use super::data::{
    Task, TaskId, TaskRequest, Workflow, WorkflowCreateRequest, WorkflowDeprecationRequest,
    WorkflowId, WorkflowUpdateRequest,
};

/// Service for fetching and interacting with workflow run data. Wraps a [Pool] and provides
/// interaction methods for the API.
pub trait WorkflowsService
where
    Self: Clone + Send,
{
    type CreateRequestValidator: ApiRequestValidator<Request = WorkflowCreateRequest>;
    type Database: Database;
    type UpdateRequestValidator: ApiRequestValidator<Request = WorkflowUpdateRequest>;

    /// Create a new workflow using the `request` data. Returns the new [Workflow] created.
    async fn create_workflow(&self, request: &WorkflowCreateRequest) -> EmResult<Workflow>;
    /// Read a single [Workflow] record for the specified `workflow_id`. Returns [Err] if the id
    /// does not match any record in the database.
    async fn read_one(&self, workflow_id: &WorkflowId) -> EmResult<Workflow>;
    /// Read all [Workflow] records in the database
    async fn read_many(&self) -> EmResult<Vec<Workflow>>;
    /// Update an existing workflow using the `request` data. Returns the new state of the
    /// [Workflow] updated.
    async fn update_workflow(&self, request: &WorkflowUpdateRequest) -> EmResult<Workflow>;
    /// Deprecate the workflow specified within the `request` data, pointing to a new workflow
    /// if the `request` contains a `new_workflow_id` value. Returns the `workflow_id` that was
    /// updated as a response.
    async fn deprecate(&self, request: &WorkflowDeprecationRequest) -> EmResult<WorkflowId>;
}

/// Service for fetching and interacting with task data. Wraps a `pool` and provides interaction
/// methods for the API.
pub trait TaskService
where
    Self: Clone + Send,
{
    type Database: Database;
    type RequestValidator: ApiRequestValidator<Request = TaskRequest>;

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
