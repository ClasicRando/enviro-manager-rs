use std::collections::HashMap;

use common::{
    database::listener::ChangeListener,
    error::{EmError, EmResult},
};
use log::{error, info, warn};
use tokio::{signal::ctrl_c, task::JoinError};

use super::{
    data::{ExecutorId, ExecutorStatus},
    service::ExecutorService,
    utilities::{
        ExecutorStatusUpdate, WorkflowRunCancelMessage, WorkflowRunScheduledMessage,
        WorkflowRunWorkerResult,
    },
};
use crate::workflow_run::{
    data::{ExecutorWorkflowRun, TaskQueueRecord, WorkflowRunId, WorkflowRunStatus},
    service::{TaskQueueService, WorkflowRunsService},
};

/// Next operations available to an [Executor] after performing various checks on the status of
/// listeners, queues and signals.
///
/// [Continue][ExecutorNextOperation::Continue] and [Break][ExecutorNextOperation::Break] are both
/// results of a notification sent to the executor using the built-in LISTEN/NOTIFY system from
/// postgresql. They represent a required operation of the same name be applied to the main loop
/// of the [Executor]. [Break][ExecutorNextOperation::Break] contains the notification signal type
/// that was sent to cause a break to happen to dictate the kind of shutdown to occur (forced or
/// graceful).
///
/// [NextWorkflowRun][ExecutorNextOperation::NextWorkflowRun] occurs when a workflow run is
/// available for the [Executor] to run. This variant is provided once the workflow run has
/// started and the [WorkflowRunId] and [JoinHandle] are returned within the variant.
///
/// [Listen][ExecutorNextOperation::Listen] occurs when no new workflows are available to process
/// and the executor should move into standby mode. This means the executor is only listening for
/// wake-up notifications are a SIGINT signal.
enum ExecutorNextOperation {
    Continue,
    Break(ExecutorStatusUpdate),
    NextWorkflowRun(WorkflowRunId, WorkflowRunWorkerResult),
    Listen,
}

/// Main unit of work for the workflow engine. Manages
/// [WorkflowRunWorker][crate::executor::WorkflowRunWorker] instances that are delegated to the
/// [Executor] instance. Operates through the creation of an [Executor] using [Executor::new],
/// followed by a call to [Executor::run] to allow the [Executor] to operate and pick up new
/// workflow runs. Under the hood, the [Executor] spawns tokio [tasks][tokio::spawn] to handle each
/// workflow run, calling the linked task service for the current task.
///
/// Running of an [Executor] works in 2 stages, changing based upon notification or data available
/// from the database. The first mode is the active mode. During this operation, the [Executor] is
/// listening for changes in executor status, shutdown signals from the application runner (ctrl+c)
/// and workflow run cancel notification. It is also checking for new workflow runs to claim and
/// run. If no workflow runs are available, the [Executor] enters a listen mode where it still
/// listens for the previous notifications/signals but also listens for new workflow runs scheduled
/// for pick-up.
///
/// After the [Executor] has completed it's run (either through graceful shutdown, cancel or error)
/// the [Executor] enters shutdown and cleaning mode to free workflow runs that are currently in
/// progress (if any). After cleaning all relevant resources, the [Executor] instance is dropped to
/// avoid any issues with held services or workflow run handles.
pub struct Executor<E, W, T>
where
    E: ExecutorService,
    W: WorkflowRunsService,
    T: TaskQueueService,
{
    executor_id: ExecutorId,
    executor_service: E,
    wr_service: W,
    tq_service: T,
    wr_handles: HashMap<WorkflowRunId, WorkflowRunWorkerResult>,
}

impl<U, C, S, E, W, T> Executor<E, W, T>
where
    U: ChangeListener<Message = ExecutorStatusUpdate>,
    C: ChangeListener<Message = WorkflowRunCancelMessage>,
    S: ChangeListener<Message = WorkflowRunScheduledMessage>,
    E: ExecutorService<Listener = U>,
    W: WorkflowRunsService<CancelListener = C, ScheduledListener = S>,
    T: TaskQueueService,
{
    /// Create a new [Executor] using the provided services. Cleans output unused or stale
    /// executors in the database before registering the current [Executor] and returning the new
    /// [Executor].
    /// # Errors
    /// This function will return an error if either the cleaning of existing executors or the
    /// registering of this new executor fails.
    pub async fn new(executor_service: &E, wr_service: &W, tq_service: &T) -> EmResult<Self> {
        executor_service.clean_executors().await?;
        let executor_id = executor_service.register_executor().await?;
        Ok(Self {
            executor_id,
            executor_service: executor_service.clone(),
            wr_service: wr_service.clone(),
            tq_service: tq_service.clone(),
            wr_handles: HashMap::new(),
        })
    }

    /// Return a reference to the [ExecutorId] of the [Executor].
    pub const fn executor_id(&self) -> &ExecutorId {
        &self.executor_id
    }

    pub async fn run(mut self) {
        if let Err(error) = self.run_loop().await {
            self.executor_service
                .post_error(&self.executor_id, error)
                .await;
        }
    }

    #[allow(unused_assignments)]
    async fn run_loop(&mut self) -> EmResult<()> {
        let mut is_listen_mode = false;
        let mut executor_signal: ExecutorStatusUpdate;
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
                    executor_signal = ExecutorStatusUpdate::Cancel;
                    break;
                }
                ExecutorStatus::Shutdown => {
                    executor_signal = ExecutorStatusUpdate::Shutdown;
                    break;
                }
            }
            self.cleanup_workflows().await?;

            let next_operation = if is_listen_mode {
                info!("Starting listen mode.");
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
                ExecutorNextOperation::Continue => {
                    is_listen_mode = false;
                    executor_signal = ExecutorStatusUpdate::NoOp;
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

    /// Add a new workflow run handle for the executor. Stored for checking if the spawned tokio
    /// task is complete during cleanup scans.
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

    /// Read the current status of the executor as stored by the database
    async fn status(&self) -> EmResult<ExecutorStatus> {
        self.executor_service.read_status(&self.executor_id).await
    }

    /// Handle a manual shutdown by the user (performed by a ctrl+c) by logging the even and
    /// returning a [ExecutorNextOperation::Break] entry containing a
    /// [ExecutorNotificationSignal::Shutdown] variant.
    fn handle_manual_shutdown() -> ExecutorNextOperation {
        info!("Received shutdown signal. Starting graceful shutdown");
        ExecutorNextOperation::Break(ExecutorStatusUpdate::Shutdown)
    }

    /// Select the next operation when in the active state of an executor. 1 of 4 operations are
    /// awaited for first completion (priority given respective to order):
    /// - ctrl+c
    /// - executor status notification
    /// - workflow run cancel notification
    /// - next workflow run available polled
    ///
    /// Whichever operation completes first will handle the completed future and return an
    /// [ExecutorNextOperation] variant to tell the executor what to do as the next step.
    async fn next_operation_active(
        &mut self,
        executor_status_listener: &mut U,
        workflow_run_cancel_listener: &mut C,
    ) -> EmResult<ExecutorNextOperation> {
        Ok(tokio::select! {
            biased;
            _ = ctrl_c() => Self::handle_manual_shutdown(),
            notification = executor_status_listener.recv() => Self::
                handle_executor_status_notification(notification?),
            notification = workflow_run_cancel_listener.recv() => self
                .handle_workflow_run_cancel_notification(notification?).await?,
            workflow_run_id = self.next_workflow_run() => {
                let Some((workflow_run_id, run_result)) = workflow_run_id? else {
                    return Ok(ExecutorNextOperation::Listen)
                };
                ExecutorNextOperation::NextWorkflowRun(workflow_run_id, run_result)
            }
        })
    }

    /// Select the next operation when in the listen state of an executor. 1 of 4 operations are
    /// awaited for first completion (priority given respective to order):
    /// - ctrl+c
    /// - executor status notification
    /// - workflow run cancel notification
    /// - workflow run scheduled notification
    ///
    /// Whichever operation completes first will handle the completed future and return an
    /// [ExecutorNextOperation] variant to tell the executor what to do as the next step.
    async fn next_operation_listen(
        &mut self,
        executor_status_listener: &mut U,
        workflow_run_cancel_listener: &mut C,
        workflow_run_scheduled_listener: &mut S,
    ) -> EmResult<ExecutorNextOperation> {
        Ok(tokio::select! {
            biased;
            _ = ctrl_c() => Self::handle_manual_shutdown(),
            notification = executor_status_listener.recv() => Self::
                handle_executor_status_notification(notification?),
            notification = workflow_run_cancel_listener.recv() => self
                .handle_workflow_run_cancel_notification(notification?).await?,
            notification = workflow_run_scheduled_listener.recv() => Self::
                handle_workflow_run_scheduled_notification(notification)?,
        })
    }

    /// Fetch the next available workflow run for the current executor.
    ///
    /// If there is an available workflow run, a workflow run worker is spawned and returned with
    /// the workflow run id. If no workflow run is available, the function immediately returns with
    /// a wrapped [None] value.
    async fn next_workflow_run(
        &self,
    ) -> EmResult<Option<(WorkflowRunId, WorkflowRunWorkerResult)>> {
        let Some(workflow_run_id) = self.executor_service.next_workflow_run(&self.executor_id).await? else {
            return Ok(None)
        };
        let wr_handle = self.spawn_workflow_run_worker(&workflow_run_id);
        Ok(Some((workflow_run_id, wr_handle)))
    }

    /// Spawn a new tokio task to execute the workflow run of the `workflow_run_id` provided.
    /// Returns the [JoinHandle][tokio::task::JoinHandle] to the spawned task.
    fn spawn_workflow_run_worker(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> WorkflowRunWorkerResult {
        let wr_service = self.wr_service.clone();
        let tq_service = self.tq_service.clone();
        let workflow_run_id = *workflow_run_id;
        tokio::spawn(async move {
            let worker = WorkflowRunWorker::new(workflow_run_id, wr_service, tq_service);
            let worker_result = worker.run().await;

            let mut err = None;
            if let Err(error) = worker_result {
                error!("WE Error\n{:?}", error);
                err = Some(error);
            }
            (workflow_run_id, err)
        })
    }

    /// Handle [JoinError] returned when a tokio task does not complete successfully when joined.
    /// Simply logs and discards the error.
    fn handle_join_error(workflow_run_id: &WorkflowRunId, error: &JoinError) {
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

    /// Handle a notification through the workflow run cancel channel. Attempts to parse the
    /// notification body into a workflow run id, pulling the respective workflow run handle (if
    /// any), aborting is not finished, finally joining the task and cancelling the workflow run
    /// through the database service.
    async fn handle_workflow_run_cancel_notification(
        &mut self,
        message: WorkflowRunCancelMessage,
    ) -> EmResult<ExecutorNextOperation> {
        let WorkflowRunCancelMessage(Some(workflow_run_id)) = message else {
            return Ok(ExecutorNextOperation::Continue)
        };
        let Some(handle) = self.wr_handles.remove(&workflow_run_id) else {
            return Ok(ExecutorNextOperation::Continue)
        };

        if !handle.is_finished() {
            handle.abort();
        }

        if let Err(error) = handle.await {
            Self::handle_join_error(&workflow_run_id, &error)
        }
        self.wr_service.cancel(&workflow_run_id).await?;
        Ok(ExecutorNextOperation::Continue)
    }

    /// Handle a notification through the workflow run scheduled channel. If the notification is
    /// received successfully, the executor is told to start the loop over again, exiting listen
    /// mode to handle new workflow runs.
    fn handle_workflow_run_scheduled_notification(
        result: EmResult<WorkflowRunScheduledMessage>,
    ) -> EmResult<ExecutorNextOperation> {
        match result {
            Ok(_) => {
                info!("Notification of workflow run scheduled. Starting loop again.");
                Ok(ExecutorNextOperation::Continue)
            }
            Err(error) => {
                error!("Error receiving workflow run notification.\n{:?}", error);
                Err(error)
            }
        }
    }

    /// Handle a notification through the executor status channel. Parse the notification body into
    /// an [ExecutorNotificationSignal], returning a [ExecutorNextOperation::Break] signal if the
    /// notification payload matches a [ExecutorNotificationSignal::Cancel] or
    /// [ExecutorNotificationSignal::Shutdown] signal.
    const fn handle_executor_status_notification(
        status_update: ExecutorStatusUpdate,
    ) -> ExecutorNextOperation {
        match &status_update {
            ExecutorStatusUpdate::Cancel | ExecutorStatusUpdate::Shutdown => {
                ExecutorNextOperation::Break(status_update)
            }
            ExecutorStatusUpdate::NoOp => ExecutorNextOperation::Continue,
        }
    }

    /// Process a workflow run that is owned by the current executor, but missing a workflow run
    /// handle. Handles 3 cases:
    /// - workflow run if invalid - cancel workflow run and exit
    /// - workflow run has status of 'Running' - spawn workflow, add handle and exit
    /// - else - restart the workflow run and schedule for the current executor
    async fn process_unknown_run(&mut self, workflow_run: ExecutorWorkflowRun) -> EmResult<()> {
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

    /// Clean the workflows of the current executor. Checks completed handles to remove from map
    /// to free resources. Also checks for owned workflows that have no handle. For each unknown
    /// workflow run, [process_unknown_run][Executor::process_unknown_run] is called to fix the
    /// run.
    async fn cleanup_workflows(&mut self) -> EmResult<()> {
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

    /// Complete a workflow run handle when attempting to shutdown workers. Checks the handle to
    /// see if it is already completed (while returning false). If the handle is not complete, and
    /// the executor is in the process of being cancelled, the handle is aborted before starting
    /// the move of the workflow run to the next available executor.
    async fn finish_handle(
        &self,
        workflow_run_id: &WorkflowRunId,
        handle: &WorkflowRunWorkerResult,
        is_cancelled: &bool,
    ) -> EmResult<bool> {
        if handle.is_finished() {
            return Ok(false);
        }

        if *is_cancelled {
            handle.abort();
            return Ok(false);
        }

        self.wr_service.start_move(workflow_run_id).await?;
        Ok(true)
    }

    /// Shutdown all workflow run workers by iterating over all workflow run handles, completing
    /// the handles based upon the completion status of the executor. If the executor is in the
    /// process of being cancelled and a handle is no finished, the executor will attempt to move
    /// the workflow run to the next available executor, falling back to a scheduled state when no
    /// other executor is operating.
    async fn shutdown_workers(&mut self, is_cancelled: &bool) -> EmResult<()> {
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
                    if *is_cancelled { "forced" } else { "graceful" },
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

    /// Close the executor by first shutting down workflow run workers then closing the executor
    /// from the database's perspective. The executor instance is dropped at the end of this
    /// function.
    async fn close_executor(&mut self, signal: ExecutorStatusUpdate) -> EmResult<()> {
        info!("Shutting down workers");
        let is_cancelled = signal.is_cancelled();
        self.shutdown_workers(&is_cancelled).await?;

        info!("Closing executor");
        self.executor_service
            .close(&self.executor_id, is_cancelled)
            .await?;
        Ok(())
    }
}

/// Container with the workflow run ID associated with the worker and the necessary services to
/// complete workflow run operations
struct WorkflowRunWorker<W, T>
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
    const fn new(workflow_run_id: WorkflowRunId, wr_service: W, tq_service: T) -> Self {
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
    async fn run(self) -> EmResult<()> {
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
