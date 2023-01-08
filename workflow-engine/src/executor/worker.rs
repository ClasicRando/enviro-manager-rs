use log::{error, info};
use sqlx::{Postgres, Transaction};

use crate::{
    error::Result as WEResult,
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

    pub async fn run(self) -> WEResult<()> {
        loop {
            let Some((next_task, transaction)) = self.next_task().await? else {
                self.wr_service.complete(self.workflow_run_id).await?;
                info!("No available task to run. Exiting worker");
                break;
            };
            info!("Running task, {:?}", next_task);
            self.tq_service.start_task_run(&next_task, transaction).await?;
            match self.tq_service.run_task(&next_task).await {
                Ok((is_paused, output)) => {
                    self.tq_service
                        .complete_task_run(&next_task, is_paused, output)
                        .await?;
                }
                Err(error) => {
                    self.tq_service.fail_task_run(&next_task, error).await?;
                    self.wr_service.complete(self.workflow_run_id).await?;
                    error!("Task failed, {:?}", next_task);
                    break;
                }
            }
        }
        Ok(())
    }
}
