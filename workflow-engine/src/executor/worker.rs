use common::error::{EmError, EmResult};
use log::{error, info};

use crate::workflow_run::{
    data::{TaskQueueRecord, WorkflowRunId},
    service::{TaskQueueService, WorkflowRunsService},
};

/// Container with the workflow run ID associated with the worker and the necessary services to
/// complete workflow run operations
pub struct WorkflowRunWorker<W, T>
where
    W: WorkflowRunsService,
    T: TaskQueueService,
{
    workflow_run_id: WorkflowRunId,
    wr_service: W,
    tq_service: T,
}

impl<W, T> WorkflowRunWorker<W, T>
where
    W: WorkflowRunsService,
    T: TaskQueueService,
{
    /// Create a new worker. Worker does nothing until [`WorkflowRunWorker::run`] is called.
    ///
    /// # Arguments
    ///
    /// * `workflow_run_id` - ID of the workflow run to be executed
    /// * `wr_service` - workflow run service to interact with the database
    /// * `tq_service` - task queue service to interact with the database
    pub const fn new(workflow_run_id: WorkflowRunId, wr_service: W, tq_service: T) -> Self {
        Self {
            workflow_run_id,
            wr_service,
            tq_service,
        }
    }

    /// Complete a task run, updating the database record with run results
    async fn complete_task(
        &self,
        record: &TaskQueueRecord,
        is_paused: bool,
        message: Option<String>,
    ) -> EmResult<()> {
        self.tq_service
            .complete_task_run(record, is_paused, message)
            .await
    }

    /// Fail the task run, updating the database record with error information
    async fn fail_task(&self, record: &TaskQueueRecord, error: EmError) -> EmResult<()> {
        error!("Task failed, {:?}", record);
        self.tq_service.fail_task_run(record, error).await?;
        self.wr_service.complete(&self.workflow_run_id).await
    }

    /// Entry point for running the worker. Continues to get the next task to run until no more
    /// tasks are available or a task fails. Once this is completed, the worker is dropped.
    pub async fn run(self) -> EmResult<()> {
        loop {
            let Some(next_task) = self.tq_service.next_task(&self.workflow_run_id).await? else {
                self.wr_service.complete(&self.workflow_run_id).await?;
                info!("No available task to run. Exiting worker");
                break;
            };
            info!("Running task, {:?}", next_task);
            match self.tq_service.run_task(&next_task).await {
                Ok((is_paused, message)) => {
                    self.complete_task(&next_task, is_paused, message).await?
                }
                Err(error) => {
                    self.fail_task(&next_task, error).await?;
                    break;
                }
            }
        }
        Ok(())
    }
}
