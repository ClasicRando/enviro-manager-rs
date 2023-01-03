mod utilities;
mod worker;

use std::collections::HashMap;

use log::{error, info, warn};
use sqlx::{postgres::PgNotification, Error as SqlError};
use tokio::signal::ctrl_c;
use utilities::{ExecutorNotificationSignal, WorkflowRunWorkerResult};
use worker::WorkflowRunWorker;

use crate::{
    error::Result as WEResult,
    services::{
        executors::{ExecutorStatus, ExecutorsService},
        task_queue::TaskQueueService,
        workflow_runs::{WorkflowRunStatus, WorkflowRunsService},
    },
};

pub struct Executor {
    executor_id: i64,
    executor_service: &'static ExecutorsService,
    wr_service: &'static WorkflowRunsService,
    tq_service: &'static TaskQueueService,
    wr_handles: HashMap<i64, WorkflowRunWorkerResult>,
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

    fn add_workflow_run_handle(&mut self, workflow_run_id: i64, handle: WorkflowRunWorkerResult) {
        self.wr_handles.insert(workflow_run_id, handle);
        info!(
            "Created sub executor to handle workflow_run_id = {}",
            workflow_run_id
        );
    }

    async fn status(&self) -> WEResult<ExecutorStatus> {
        self.executor_service
            .read_status(self.executor_id)
            .await
            .map(|status| status.unwrap_or(ExecutorStatus::Canceled))
    }

    pub async fn run(&mut self) -> WEResult<()> {
        let mut executor_signal: ExecutorNotificationSignal;
        let mut executor_status_listener = self
            .executor_service
            .status_listener(self.executor_id)
            .await?;
        let mut workflow_run_scheduled_listener =
            self.wr_service.scheduled_listener(self.executor_id).await?;
        let mut workflow_run_cancel_listener =
            self.wr_service.cancel_listener(self.executor_id).await?;
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
            let workflow_run: Option<(i64, WorkflowRunWorkerResult)> = tokio::select! {
                biased;
                _ = ctrl_c() => {
                    info!("Received shutdown signal. Starting graceful shutdown");
                    executor_signal = ExecutorNotificationSignal::Shutdown;
                    break;
                }
                notification = executor_status_listener.recv() => {
                    executor_signal = self.handle_executor_status_notification(notification);
                    match &executor_signal {
                        ExecutorNotificationSignal::Cancel
                        | ExecutorNotificationSignal::Shutdown
                        | ExecutorNotificationSignal::Error(_) => break,
                        ExecutorNotificationSignal::Cleanup
                        | ExecutorNotificationSignal::NoOp => continue,
                    }
                }
                notification = workflow_run_cancel_listener.recv() => {
                    self.handle_workflow_run_cancel_notification(notification).await?;
                    continue;
                }
                workflow_run_id = self.next_workflow() => {
                    workflow_run_id?
                }
            };
            match workflow_run {
                Some((workflow_run_id, handle)) => {
                    self.add_workflow_run_handle(workflow_run_id, handle);
                    continue;
                }
                None => {
                    info!("No more workflow runs available. Switching to listen mode.");
                    tokio::select! {
                        biased;
                        _ = ctrl_c() => {
                            info!("Received shutdown signal. Starting graceful shutdown");
                            executor_signal = ExecutorNotificationSignal::Shutdown;
                            break;
                        }
                        notification = executor_status_listener.recv() => {
                            executor_signal = self.handle_executor_status_notification(notification);
                            match &executor_signal {
                                ExecutorNotificationSignal::Cancel
                                | ExecutorNotificationSignal::Shutdown
                                | ExecutorNotificationSignal::Error(_) => break,
                                ExecutorNotificationSignal::Cleanup
                                | ExecutorNotificationSignal::NoOp => continue,
                            }
                        }
                        notification = workflow_run_cancel_listener.recv() => {
                            self.handle_workflow_run_cancel_notification(notification).await?;
                            continue;
                        }
                        notification = workflow_run_scheduled_listener.recv() => {
                            self.handle_workflow_run_scheduled_notification(notification)?;
                            continue
                        }
                    }
                }
            }
        }
        self.close_executor(executor_signal).await?;
        Ok(())
    }

    async fn next_workflow(&self) -> WEResult<Option<(i64, WorkflowRunWorkerResult)>> {
        let Some(workflow_run_id) = self.executor_service.next_workflow_run(self.executor_id).await? else {
            return Ok(None)
        };
        let wr_handle = self.spawn_workflow_run_worker(workflow_run_id);
        Ok(Some((workflow_run_id, wr_handle)))
    }

    fn spawn_workflow_run_worker(&self, workflow_run_id: i64) -> WorkflowRunWorkerResult {
        let wr_service = self.wr_service;
        let tq_service = self.tq_service;
        tokio::spawn(async move {
            let worker = WorkflowRunWorker::new(workflow_run_id, wr_service, tq_service);
            let worker_result = worker.run().await;
            match worker_result {
                Ok(_) => (workflow_run_id, None),
                Err(error) => {
                    error!("WE Error\n{:?}", error);
                    (workflow_run_id, Some(error))
                }
            }
        })
    }

    async fn handle_workflow_run_cancel_notification(
        &mut self,
        result: Result<PgNotification, SqlError>,
    ) -> WEResult<()> {
        match result {
            Ok(notification) => {
                let workflow_run_id: i64 =
                    notification.payload().parse::<i64>().unwrap_or_default();
                let workflow_run_handle = self.wr_handles.remove(&workflow_run_id);
                if let Some(handle) = workflow_run_handle {
                    if !handle.is_finished() {
                        handle.abort();
                    }
                    if let Err(error) = handle.await {
                        if error.is_cancelled() {
                            warn!("Workflow run = {} canceled\n{}", workflow_run_id, error)
                        } else if error.is_panic() {
                            error!("Workflow run = {} panicked!\n{}", workflow_run_id, error)
                        } else {
                            info!("Workflow run = {} completed\n{}", workflow_run_id, error)
                        }
                    }
                    self.wr_service.cancel(workflow_run_id).await?;
                }
            }
            Err(error) => {
                error!(
                    "Error receiving workflow run cancel notification.\n{:?}",
                    error
                );
            }
        }
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
        match result {
            Ok(notification) => {
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
            Err(error) => {
                error!("Error receiving executor notification.\n{:?}", error);
                ExecutorNotificationSignal::Error(error)
            }
        }
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
            .all_executor_workflows(self.executor_id)
            .await?;
        for wr in workflow_runs {
            if self.wr_handles.contains_key(&wr.workflow_run_id) {
                continue;
            }

            if wr.is_valid {
                info!("Restarting workflow_run_id = {}", wr.workflow_run_id);
                if wr.status == WorkflowRunStatus::Running {
                    let wr_handle = self.spawn_workflow_run_worker(wr.workflow_run_id);
                    self.add_workflow_run_handle(wr.workflow_run_id, wr_handle);
                } else {
                    self.wr_service.restart(wr.workflow_run_id).await?;
                    self.wr_service
                        .schedule_with_executor(wr.workflow_run_id, self.executor_id)
                        .await?;
                }
            } else {
                info!("Canceling workflow_run_id = {}", wr.workflow_run_id);
                self.wr_service.cancel(wr.workflow_run_id).await?;
            }
        }
        Ok(())
    }

    async fn shutdown_workers(&mut self, is_forced: bool) -> WEResult<bool> {
        let handle_keys: Vec<i64> = self.wr_handles.keys().map(|key| key.to_owned()).collect();
        for workflow_run_id in handle_keys {
            let Some(handle) = self.wr_handles.remove(&workflow_run_id) else {
                continue;
            };

            let is_move = if !handle.is_finished() {
                if is_forced {
                    handle.abort();
                    false
                } else {
                    self.wr_service.start_move(workflow_run_id).await?;
                    true
                }
            } else {
                false
            };

            if let Err(join_error) = handle.await {
                warn!(
                    "Join error during {} shutdown.\n{}",
                    if is_forced { "forced" } else { "graceful" },
                    join_error
                );
            }

            if is_move {
                self.wr_service.complete_move(workflow_run_id).await?;
            } else {
                self.wr_service.cancel(workflow_run_id).await?;
            }
        }
        Ok(is_forced)
    }

    async fn close_executor(&mut self, signal: ExecutorNotificationSignal) -> WEResult<()> {
        info!("Shutting down workers");
        let is_cancelled = match signal {
            ExecutorNotificationSignal::Cancel | ExecutorNotificationSignal::Error(_) => {
                self.shutdown_workers(true).await?
            }
            ExecutorNotificationSignal::Shutdown
            | ExecutorNotificationSignal::NoOp
            | ExecutorNotificationSignal::Cleanup => self.shutdown_workers(false).await?,
        };

        info!("Closing executor");
        self.executor_service
            .close(self.executor_id, is_cancelled)
            .await?;
        Ok(())
    }
}
