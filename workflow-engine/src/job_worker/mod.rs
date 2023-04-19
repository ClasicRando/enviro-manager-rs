use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use log::{error, info, warn};
use sqlx::postgres::PgNotification;
use tokio::{
    signal::ctrl_c,
    time::{sleep as tokio_sleep, Duration as StdDuration},
};

use crate::{
    services::jobs::{Job, JobId, JobsService},
    Error as WEError, Result as WEResult,
};

/// Action to perform after receiving a job worker notification. Notification payload should be a
/// workflow run id (as an i64/bigint) to tell the job worker a job has been completed or an empty
/// payload to tell the worker to refresh the job list.
enum NotificationAction {
    LoadJobs,
    CompleteJob(JobId),
}

impl TryFrom<PgNotification> for NotificationAction {
    type Error = WEError;

    fn try_from(value: PgNotification) -> Result<Self, Self::Error> {
        let payload = value.payload();
        if payload.is_empty() {
            return Ok(NotificationAction::LoadJobs);
        }
        let Ok(job_id) = payload.parse::<i64>() else {
            return Err(WEError::PayloadParseError(payload.to_owned()))
        };
        info!("Received notification of \"{}\"", payload);
        Ok(NotificationAction::CompleteJob(job_id.into()))
    }
}

/// Main unit of the recurring job run process. An instance of the worker is meant to be created
/// and run as the lifecycle of the instance (dropped at the end of the  method).
pub struct JobWorker {
    service: &'static JobsService,
    jobs: HashMap<JobId, NaiveDateTime>,
    next_job: JobId,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl JobWorker {
    /// Create a new job worker, initializing with a reference to a [JobsService] and creating a
    /// mailer to send job related emails to maintainers.
    pub async fn new(service: &'static JobsService) -> WEResult<Self> {
        let credentials = Credentials::from((env!("CLIPPY_USERNAME"), env!("CLIPPY_PASSWORD")));
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(env!("CLIPPY_RELAY"))?
            .credentials(credentials)
            .build();
        Ok(Self {
            service,
            jobs: HashMap::new(),
            next_job: 0.into(),
            mailer,
        })
    }

    /// Run the main action of the worker. Continuously listens for notification and executes the
    /// next job when ready. If there are no jobs available for the worker, it will wait for a
    /// shutdown signal (ctrl+c) or a new notification to load jobs.
    pub async fn run(mut self) -> WEResult<()> {
        let mut job_channel = self.service.listener().await?;
        self.load_jobs().await?;
        loop {
            let next_run = self
                .jobs
                .get(&self.next_job)
                .map(|next_run| {
                    let duration = next_run.timestamp_millis() - Utc::now().timestamp_millis();
                    StdDuration::from_millis(duration.clamp(0, i64::MAX) as u64)
                })
                .unwrap_or(StdDuration::MAX);
            if next_run != StdDuration::MAX {
                info!("Next run in {:?}", next_run);
            } else {
                info!("Waiting for job update notification");
            }
            tokio::select! {
                biased;
                _ = ctrl_c() => {
                    info!("Received shutdown signal. Starting graceful shutdown");
                    break;
                }
                notification = job_channel.recv() => {
                    self.handle_notification(notification?).await?
                }
                _ = tokio_sleep(next_run) => {
                    self.run_next_job().await?;
                    self.load_jobs().await?;
                }
            }
        }
        Ok(())
    }

    /// Load all available jobs from the job queue in `job.jobs`. If the job queue becomes polluted
    /// with a duplicate job id, an error will be returned (although this should never happen
    /// unless the database is corrupt/altered). Once jobs are fetched, if any jobs exist, the
    /// first available job will be queued as the next job.
    async fn load_jobs(&mut self) -> WEResult<()> {
        info!("Requesting new job queue");
        let jobs = self.service.read_queued().await?;
        self.jobs.clear();
        self.next_job = jobs.get(0).map(|j| j.job_id).unwrap_or(0).into();
        for job in jobs {
            let job_id = job.job_id.into();
            if let Some(duplicate) = self.jobs.get(&job_id) {
                return Err(WEError::DuplicateJobId(
                    job.job_id,
                    [job.next_run, duplicate.to_owned()],
                ));
            }
            self.jobs.insert(job_id, job.next_run);
        }
        Ok(())
    }

    /// Handle a notification, parsing to a [NotificationAction] and handling each action. If the
    /// notification is [NotificationAction::LoadJobs] then the jobs cache will be refreshed. If
    /// the notification is [NotificationAction::CompleteJob] the the inner `job_id` will be used
    /// to mark a job as complete and jobs list will be refreshed.
    async fn handle_notification(&mut self, notification: PgNotification) -> WEResult<()> {
        let action = match NotificationAction::try_from(notification) {
            Ok(action) => action,
            Err(error) => return Err(error),
        };
        match action {
            NotificationAction::LoadJobs => self.load_jobs().await?,
            NotificationAction::CompleteJob(job_id) => {
                self.complete_job(&job_id).await?;
                self.load_jobs().await?;
            }
        }
        Ok(())
    }

    /// Run the next job in the queue. In a usual run, the job is executed as a standalone workflow
    /// run, where the job is marked with that new workflow run id. If the run is too early an
    /// error message is printed but the worker does not fail. Instead an early exit happens and a
    /// queue refresh should follow.
    async fn run_next_job(&self) -> WEResult<()> {
        let Some(next_run) = self.jobs.get(&self.next_job) else {
            warn!("Attempted to run a job that is not in the job queue. Job_id = {}", self.next_job);
            return Ok(())
        };
        let now = Utc::now().naive_utc();
        if next_run > &now {
            error!(
                "Job was not ready. job_id = {}. Time to run = {}, current time = {}",
                self.next_job, next_run, now
            );
            return Ok(());
        }
        info!("Starting new job run for job_id = {}", self.next_job);
        self.service.run_job(&self.next_job).await?;
        Ok(())
    }

    /// Complete the specified job after the workflow run is complete. If something went wrong or
    /// the job failed, the maintainer of the job will be notified with an email. If the `job_id`
    /// is not valid then warning messages will be printed but the worker will continue.
    async fn complete_job(&self, job_id: &JobId) -> WEResult<()> {
        if !self.jobs.contains_key(job_id) {
            warn!(
                "Received a message to complete a job that is not in the job queue. Job_id = {}",
                job_id
            );
            return Ok(());
        };
        let Some(Job { maintainer, .. }) = self.service.read_one(job_id).await? else {
            warn!("Could not find a job in the database for job_id = {}", job_id);
            return Ok(())
        };
        info!("Completing run for job_id = {}", job_id);
        let Err(WEError::Generic(error)) = self.service.complete_job(job_id).await else {
            return Ok(())
        };
        self.send_error_email(maintainer, error).await?;
        Ok(())
    }

    /// Send an email to the specified `maintainer` with the error message as the email body
    async fn send_error_email(&self, maintainer: String, message: String) -> WEResult<()> {
        warn!(
            "Sending error email to {} with message\n{}",
            maintainer, message
        );
        let email = Message::builder()
            .from("Clippy".parse()?)
            .to(maintainer.parse()?)
            .subject("Job Completion Error")
            .body(message)?;
        let _response = self.mailer.send(email).await?;
        Ok(())
    }
}
