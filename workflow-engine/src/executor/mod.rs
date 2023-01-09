mod utilities;
mod worker;

use std::collections::HashMap;

use log::{error, info, warn};
use sqlx::{
    postgres::{PgListener, PgNotification},
    Error as SqlError,
};
use tokio::{signal::ctrl_c, task::JoinError};
use utilities::{ExecutorNotificationSignal, WorkflowRunWorkerResult};
use worker::WorkflowRunWorker;

use crate::{
    error::Result as WEResult,
    services::{
        executors::{ExecutorId, ExecutorStatus, ExecutorsService},
        task_queue::TaskQueueService,
        workflow_runs::{
            ExecutorWorkflowRun, WorkflowRunId, WorkflowRunStatus, WorkflowRunsService,
        },
    },
};

enum ExecutorNextOperation {
    Continue(ExecutorNotificationSignal),
    Break(ExecutorNotificationSignal),
    NextWorkflowRun(WorkflowRunId, WorkflowRunWorkerResult),
    Listen,
}

pub struct Executor {
    executor_id: ExecutorId,
    executor_service: &'static ExecutorsService,
    wr_service: &'static WorkflowRunsService,
    tq_service: &'static TaskQueueService,
    wr_handles: HashMap<WorkflowRunId, WorkflowRunWorkerResult>,
}

impl Executor {
    pub async fn new(
        executor_service: &'static ExecutorsService,
        wr_service: &'static WorkflowRunsService,
        tq_service: &'static TaskQueueService,
    ) -> WEResult<Self> {
        executor_service.clean_executors().await?;
        let executor_id = executor_service.register_executor().await?;
        Ok(Self {
            executor_id,
            executor_service,
            wr_service,
            tq_service,
            wr_handles: HashMap::new(),
        })
    }

    pub fn executor_id(&self) -> &ExecutorId {
        &self.executor_id
    }

    fn add_workflow_run_handle(
        &mut self,
        workflow_run_id: WorkflowRunId,
        handle: WorkflowRunWorkerResult,
    ) {
        info!(
            "Created sub executor to handle workflow_run_id = {}",
            workflow_run_id
        );
        self.wr_handles.insert(workflow_run_id, handle);
    }

    async fn status(&self) -> WEResult<ExecutorStatus> {
        self.executor_service
            .read_status(&self.executor_id)
            .await
            .map(|status| status.unwrap_or(ExecutorStatus::Canceled))
    }

    async fn next_operation_active(
        &mut self,
        executor_status_listener: &mut PgListener,
        workflow_run_cancel_listener: &mut PgListener,
    ) -> WEResult<ExecutorNextOperation> {
        Ok(tokio::select! {
            biased;
            _ = ctrl_c() => {
                info!("Received shutdown signal. Starting graceful shutdown");
                ExecutorNextOperation::Break(ExecutorNotificationSignal::Shutdown)
            }
            notification = executor_status_listener.recv() => {
                let executor_signal = self.handle_executor_status_notification(notification);
                match &executor_signal {
                    ExecutorNotificationSignal::Cancel
                    | ExecutorNotificationSignal::Shutdown
                    | ExecutorNotificationSignal::Error(_) => ExecutorNextOperation::Break(executor_signal),
                    ExecutorNotificationSignal::Cleanup
                    | ExecutorNotificationSignal::NoOp => ExecutorNextOperation::Continue(executor_signal),
                }
            }
            notification = workflow_run_cancel_listener.recv() => {
                self.handle_workflow_run_cancel_notification(notification).await?;
                ExecutorNextOperation::Continue(ExecutorNotificationSignal::NoOp)
            }
            workflow_run_id = self.next_workflow() => {
                let Some((workflow_run_id, run_result)) = workflow_run_id? else {
                    return Ok(ExecutorNextOperation::Listen)
                };
                ExecutorNextOperation::NextWorkflowRun(workflow_run_id, run_result)
            }
        })
    }

    async fn next_operation_listen(
        &mut self,
        executor_status_listener: &mut PgListener,
        workflow_run_cancel_listener: &mut PgListener,
        workflow_run_scheduled_listener: &mut PgListener,
    ) -> WEResult<ExecutorNextOperation> {
        Ok(tokio::select! {
            biased;
            _ = ctrl_c() => {
                info!("Received shutdown signal. Starting graceful shutdown");
                ExecutorNextOperation::Break(ExecutorNotificationSignal::Shutdown)
            }
            notification = executor_status_listener.recv() => {
                let executor_signal = self.handle_executor_status_notification(notification);
                match &executor_signal {
                    ExecutorNotificationSignal::Cancel
                    | ExecutorNotificationSignal::Shutdown
                    | ExecutorNotificationSignal::Error(_) => ExecutorNextOperation::Break(executor_signal),
                    ExecutorNotificationSignal::Cleanup
                    | ExecutorNotificationSignal::NoOp => ExecutorNextOperation::Continue(executor_signal),
                }
            }
            notification = workflow_run_cancel_listener.recv() => {
                self.handle_workflow_run_cancel_notification(notification).await?;
                ExecutorNextOperation::Continue(ExecutorNotificationSignal::NoOp)
            }
            notification = workflow_run_scheduled_listener.recv() => {
                self.handle_workflow_run_scheduled_notification(notification)?;
                ExecutorNextOperation::Continue(ExecutorNotificationSignal::NoOp)
            }
        })
    }

    #[allow(unused_assignments)]
    pub async fn run(&mut self) -> WEResult<()> {
        let mut is_listen_mode = false;
        let mut executor_signal: ExecutorNotificationSignal;
        let mut executor_status_listener = self
            .executor_service
            .status_listener(&self.executor_id)
            .await?;
        let mut workflow_run_scheduled_listener = self
            .wr_service
            .scheduled_listener(&self.executor_id)
            .await?;
        let mut workflow_run_cancel_listener =
            self.wr_service.cancel_listener(&self.executor_id).await?;
        loop {
            match self.status().await? {
                ExecutorStatus::Active => {}
                ExecutorStatus::Canceled => {
                    executor_signal = ExecutorNotificationSignal::Cancel;
                    break;
                }
                ExecutorStatus::Shutdown => {
                    executor_signal = ExecutorNotificationSignal::Shutdown;
                    break;
                }
            }
            self.cleanup_workflows().await?;

            let next_operation = if is_listen_mode {
                self.next_operation_listen(
                    &mut executor_status_listener,
                    &mut workflow_run_cancel_listener,
                    &mut workflow_run_scheduled_listener,
                )
                .await?
            } else {
                self.next_operation_active(
                    &mut executor_status_listener,
                    &mut workflow_run_cancel_listener,
                )
                .await?
            };
            match next_operation {
                ExecutorNextOperation::Continue(signal) => {
                    is_listen_mode = false;
                    executor_signal = signal;
                    continue;
                }
                ExecutorNextOperation::Break(signal) => {
                    executor_signal = signal;
                    break;
                }
                ExecutorNextOperation::NextWorkflowRun(workflow_run_id, handle) => {
                    is_listen_mode = false;
                    self.add_workflow_run_handle(workflow_run_id, handle);
                    continue;
                }
                ExecutorNextOperation::Listen => {
                    is_listen_mode = true;
                    info!("No more workflow runs available. Switching to listen mode.")
                }
            }
        }
        self.close_executor(executor_signal).await?;
        Ok(())
    }

    async fn next_workflow(&self) -> WEResult<Option<(WorkflowRunId, WorkflowRunWorkerResult)>> {
        let Some(workflow_run_id) = self.executor_service.next_workflow_run(&self.executor_id).await? else {
            return Ok(None)
        };
        let wr_handle = self.spawn_workflow_run_worker(&workflow_run_id);
        Ok(Some((workflow_run_id, wr_handle)))
    }

    fn spawn_workflow_run_worker(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> WorkflowRunWorkerResult {
        let wr_service = self.wr_service;
        let tq_service = self.tq_service;
        let workflow_run_id = workflow_run_id.to_owned();
        tokio::spawn(async move {
            let worker = WorkflowRunWorker::new(&workflow_run_id, wr_service, tq_service);
            let worker_result = worker.run().await;

            let mut err = None;
            if let Err(error) = worker_result {
                error!("WE Error\n{:?}", error);
                err = Some(error);
            }
            (workflow_run_id, err)
        })
    }

    fn handle_join_error(&self, workflow_run_id: &WorkflowRunId, error: JoinError) {
        if error.is_cancelled() {
            warn!("Workflow run = {} canceled\n{}", workflow_run_id, error);
            return;
        }
        if error.is_panic() {
            error!("Workflow run = {} panicked!\n{}", workflow_run_id, error);
            return;
        }
        info!("Workflow run = {} completed\n{}", workflow_run_id, error)
    }

    async fn handle_workflow_run_cancel_notification(
        &mut self,
        result: Result<PgNotification, SqlError>,
    ) -> WEResult<()> {
        let notification = match result {
            Ok(notification) => notification,
            Err(error) => {
                error!(
                    "Error receiving workflow run cancel notification.\n{:?}",
                    error
                );
                return Err(error.into());
            }
        };
        let workflow_run_id = notification.payload().parse()?;
        let Some(handle) = self.wr_handles.remove(&workflow_run_id) else {
            return Ok(())
        };

        if !handle.is_finished() {
            handle.abort();
        }

        if let Err(error) = handle.await {
            self.handle_join_error(&workflow_run_id, error)
        }
        self.wr_service.cancel(&workflow_run_id).await?;
        Ok(())
    }

    fn handle_workflow_run_scheduled_notification(
        &self,
        result: Result<PgNotification, SqlError>,
    ) -> WEResult<()> {
        match result {
            Ok(_) => {
                info!("Notification of workflow run scheduled. Starting loop again.");
                Ok(())
            }
            Err(error) => {
                error!("Error receiving workflow run notification.\n{:?}", error);
                Err(error.into())
            }
        }
    }

    fn handle_executor_status_notification(
        &self,
        result: Result<PgNotification, SqlError>,
    ) -> ExecutorNotificationSignal {
        let notification = match result {
            Ok(notification) => notification,
            Err(error) => {
                error!("Error receiving executor notification.\n{:?}", error);
                return ExecutorNotificationSignal::Error(error);
            }
        };
        info!(
            "Received executor status notification, \"{}\"",
            notification.payload()
        );
        match notification.payload() {
            "cancel" => ExecutorNotificationSignal::Cancel,
            "shutdown" => ExecutorNotificationSignal::Shutdown,
            "cleanup" => ExecutorNotificationSignal::Cleanup,
            _ => ExecutorNotificationSignal::NoOp,
        }
    }

    async fn process_unknown_run(&mut self, workflow_run: ExecutorWorkflowRun) -> WEResult<()> {
        if !workflow_run.is_valid {
            info!(
                "Canceling workflow_run_id = {}",
                workflow_run.workflow_run_id
            );
            self.wr_service
                .cancel(&workflow_run.workflow_run_id)
                .await?;
            return Ok(());
        }

        info!(
            "Restarting workflow_run_id = {}",
            workflow_run.workflow_run_id
        );

        if workflow_run.status == WorkflowRunStatus::Running {
            let wr_handle = self.spawn_workflow_run_worker(&workflow_run.workflow_run_id);
            self.add_workflow_run_handle(workflow_run.workflow_run_id, wr_handle);
            return Ok(());
        }

        self.wr_service
            .restart(&workflow_run.workflow_run_id)
            .await?;
        self.wr_service
            .schedule_with_executor(&workflow_run.workflow_run_id, &self.executor_id)
            .await?;
        Ok(())
    }

    async fn cleanup_workflows(&mut self) -> WEResult<()> {
        info!("Checking handles");
        let completed_handle_keys = self
            .wr_handles
            .iter()
            .filter(|(_, handle)| handle.is_finished())
            .map(|(id, _)| id.to_owned())
            .collect::<Vec<_>>();
        for workflow_run_id in completed_handle_keys {
            info!(
                "Removing finished handle for workflow_run_id = {}",
                workflow_run_id
            );
            self.wr_handles.remove(&workflow_run_id);
        }

        info!("Checking owned workflows");
        let workflow_runs = self
            .wr_service
            .all_executor_workflows(&self.executor_id)
            .await?;
        for wr in workflow_runs {
            if self.wr_handles.contains_key(&wr.workflow_run_id) {
                continue;
            }

            self.process_unknown_run(wr).await?
        }
        Ok(())
    }

    async fn finish_handle(
        &self,
        workflow_run_id: &WorkflowRunId,
        handle: &WorkflowRunWorkerResult,
        is_cancelled: bool,
    ) -> WEResult<bool> {
        if handle.is_finished() {
            return Ok(false);
        }

        if is_cancelled {
            handle.abort();
            return Ok(false);
        }

        self.wr_service.start_move(workflow_run_id).await?;
        Ok(true)
    }

    async fn shutdown_workers(&mut self, is_cancelled: bool) -> WEResult<()> {
        let handle_keys: Vec<WorkflowRunId> =
            self.wr_handles.keys().map(|key| key.to_owned()).collect();
        for workflow_run_id in handle_keys {
            let Some(handle) = self.wr_handles.remove(&workflow_run_id) else {
                continue;
            };

            let is_move = self
                .finish_handle(&workflow_run_id, &handle, is_cancelled)
                .await?;

            if let Err(join_error) = handle.await {
                warn!(
                    "Join error during {} shutdown.\n{}",
                    if is_cancelled { "forced" } else { "graceful" },
                    join_error
                );
            }

            if is_move {
                self.wr_service.complete_move(&workflow_run_id).await?;
                continue;
            }
            self.wr_service.cancel(&workflow_run_id).await?;
        }
        Ok(())
    }

    async fn close_executor(&mut self, signal: ExecutorNotificationSignal) -> WEResult<()> {
        info!("Shutting down workers");
        let is_cancelled = signal.is_cancelled();
        self.shutdown_workers(is_cancelled).await?;

        info!("Closing executor");
        self.executor_service
            .close(&self.executor_id, is_cancelled)
            .await?;
        Ok(())
    }
}
