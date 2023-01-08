use log::{error, info};

use crate::{
    error::Result as WEResult,
    services::{
        task_queue::TaskQueueService,
        workflow_runs::{WorkflowRunId, WorkflowRunsService},
    },
};

pub struct WorkflowRunWorker {
    workflow_run_id: WorkflowRunId,
    wr_service: &'static WorkflowRunsService,
    tq_service: &'static TaskQueueService,
}

impl WorkflowRunWorker {
    pub fn new(
        workflow_run_id: i64,
        wr_service: &'static WorkflowRunsService,
        tq_service: &'static TaskQueueService,
    ) -> Self {
        Self {
            workflow_run_id: workflow_run_id.into(),
            wr_service,
            tq_service,
        }
    }

    pub async fn run(self) -> WEResult<()> {
        loop {
            let (next_task, mut transaction) =
                match self.tq_service.next_task(&self.workflow_run_id).await? {
                    Some(task) => task,
                    None => {
                        self.wr_service.complete(&self.workflow_run_id).await?;
                        info!("No available task to run. Exiting worker");
                        break;
                    }
                };
            info!(
                "Running task, workflow_run_id = {}, task_order = {}",
                self.workflow_run_id, next_task.task_order,
            );
            sqlx::query("call start_task_run($1, $2)")
                .bind(next_task.workflow_run_id)
                .bind(next_task.task_order)
                .execute(&mut transaction)
                .await?;
            transaction.commit().await?;
            match self.tq_service.run_task(&next_task).await {
                Ok((is_paused, output)) => {
                    self.tq_service
                        .complete_task_run(&next_task, is_paused, output)
                        .await?;
                }
                Err(error) => {
                    self.tq_service.fail_task_run(&next_task, error).await?;
                    self.wr_service.complete(&self.workflow_run_id).await?;
                    error!(
                        "Task failed, workflow_run_id = {}, task_order = {}. Exiting worker",
                        self.workflow_run_id, next_task.task_order,
                    );
                    break;
                }
            }
        }
        Ok(())
    }
}
