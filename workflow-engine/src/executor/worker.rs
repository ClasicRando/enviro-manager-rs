use log::{error, info};
use sqlx::{Postgres, Transaction};

use crate::{
    error::{Error as WEError, Result as WEResult},
    services::{
        task_queue::{TaskQueueRecord, TaskQueueService},
        workflow_runs::{WorkflowRunId, WorkflowRunsService},
    },
};

pub struct WorkflowRunWorker<'w> {
    workflow_run_id: &'w WorkflowRunId,
    wr_service: &'static WorkflowRunsService,
    tq_service: &'static TaskQueueService,
}

impl<'w> WorkflowRunWorker<'w> {
    pub fn new(
        workflow_run_id: &'w WorkflowRunId,
        wr_service: &'static WorkflowRunsService,
        tq_service: &'static TaskQueueService,
    ) -> Self {
        Self {
            workflow_run_id,
            wr_service,
            tq_service,
        }
    }

    async fn next_task(&self) -> WEResult<Option<(TaskQueueRecord, Transaction<'_, Postgres>)>> {
        match self.tq_service.next_task(self.workflow_run_id).await? {
            Some(task) => Ok(Some(task)),
            None => Ok(None),
        }
    }

    async fn complete_task(
        &self,
        record: &TaskQueueRecord,
        is_paused: bool,
        message: Option<String>,
    ) -> WEResult<()> {
        self.tq_service
            .complete_task_run(record, is_paused, message)
            .await
    }

    async fn fail_task(&self, record: &TaskQueueRecord, error: WEError) -> WEResult<()> {
        error!("Task failed, {:?}", record);
        self.tq_service.fail_task_run(record, error).await?;
        self.wr_service.complete(self.workflow_run_id).await
    }

    pub async fn run(self) -> WEResult<()> {
        loop {
            let Some((next_task, transaction)) = self.next_task().await? else {
                self.wr_service.complete(self.workflow_run_id).await?;
                info!("No available task to run. Exiting worker");
                break;
            };
            info!("Running task, {:?}", next_task);
            self.tq_service
                .start_task_run(&next_task, transaction)
                .await?;
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
