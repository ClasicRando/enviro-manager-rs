pub mod postgres;

use common::{
    api::ApiRequestValidator,
    database::{listener::ChangeListener, Database},
    error::EmResult,
};

use super::data::{Job, JobId, JobMin, JobRequest};
use crate::{job_worker::NotificationAction, workflow_run::service::WorkflowRunsService};

/// Service for fetching and interacting with task data. Wraps a [PgPool] and provides
/// interaction methods for the API and [JobWorker][crate::job_worker::JobWorker].
pub trait JobService
where
    Self: Clone + Send,
{
    type CreateRequestValidator: ApiRequestValidator<Request = JobRequest>;
    type Database: Database;
    type Listener: ChangeListener<Message = NotificationAction>;
    type WorkflowRunService: WorkflowRunsService<Database = Self::Database>;

    /// Create a new [JobService] with the referenced pool as the data source
    fn create(
        pool: &<Self::Database as Database>::ConnectionPool,
        workflow_runs_service: &Self::WorkflowRunService,
    ) -> Self;
    /// Create a new job with the data contained within `request`. Branches to specific calls for
    /// [JobType::Scheduled] and [JobType::Interval].
    async fn create_job(&self, request: &JobRequest) -> EmResult<Job>;
    /// Read a single job record from `job.v_jobs` for the specified `job_id`. Will return [Err]
    /// when the id does not match a record
    async fn read_one(&self, job_id: &JobId) -> EmResult<Job>;

    /// Read all job records found from `job.v_jobs`
    async fn read_many(&self) -> EmResult<Vec<Job>>;
    /// Read all job records from `job.v_queued_jobs`. This excludes all job entries that are
    /// paused or currently have a workflow run that not complete. Ordered by the `next_run` field
    async fn read_queued(&self) -> EmResult<Vec<JobMin>>;
    /// Run the job specified by the `job_id`. Returns the [Job] entry if the `job_id` matches a
    /// record
    async fn run_job(&self, job_id: &JobId) -> EmResult<Job>;
    /// Complete the job specified by the `job_id`. Returns the [Job] entry if the `job_id` matches
    /// a record
    async fn complete_job(&self, job_id: &JobId) -> EmResult<Job>;
    /// Get a [ChangeListener] for updates on the job queue this service is watching.
    async fn listener(&self) -> EmResult<Self::Listener>;
}
