use std::{collections::HashMap, env, str::FromStr};

use chrono::{NaiveDateTime, Utc};
use common::{
    database::listener::ChangeListener,
    error::{EmError, EmResult},
};
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use log::{error, info, warn};
use tokio::{
    signal::ctrl_c,
    time::{sleep as tokio_sleep, Duration as StdDuration},
};

use crate::services::jobs::{JobId, JobService};

/// Action to perform after receiving a job worker notification. Notification payload should be a
/// workflow run id (as an i64/bigint) to tell the job worker a job has been completed or an empty
/// payload to tell the worker to refresh the job list.
pub enum NotificationAction {
    LoadJobs,
    CompleteJob(JobId),
}

impl FromStr for NotificationAction {
    type Err = EmError;

    fn from_str(s: &str) -> EmResult<Self> {
        if s.is_empty() {
            return Ok(Self::LoadJobs);
        }
        let Ok(job_id) = s.parse::<i64>() else {
            return Err(EmError::PayloadParseError(s.to_owned()))
        };
        info!("Received notification of \"{}\"", s);
        Ok(Self::CompleteJob(job_id.into()))
    }
}

/// Main unit of the recurring job run process. An instance of the worker is meant to be created
/// and run as the lifecycle of the instance (dropped at the end of the  method).
pub struct JobWorker<J> {
    service: J,
    jobs: HashMap<JobId, NaiveDateTime>,
    next_job: JobId,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl<J> JobWorker<J>
where
    J: JobService,
{
    /// Create a new job worker, initializing with a reference to a [JobService] and creating a
    /// mailer to send job related emails to maintainers.
    /// # Errors
    /// This function will returns an error if there are missing environment variables or the SMTP
    /// transport cannot be created. Require environment variables are:
    /// - CLIPPY_USERNAME -> email service username
    /// - CLIPPY_PASSWORD -> email service password
    /// - CLIPPY_RELAY -> email service relay
    pub fn new(service: J) -> EmResult<Self> {
        let username = env::var("CLIPPY_USERNAME")?;
        let password = env::var("CLIPPY_PASSWORD")?;
        let relay = env::var("CLIPPY_RELAY")?;
        let credentials = Credentials::from((username, password));
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&relay)?
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
    /// # Errors
    /// This function will return an error if an error is returned:
    /// - creating the job change listener
    /// - loading the new job map
    /// - parsing the job listener notification
    /// - handling the job listener notification
    /// - running the next job
    pub async fn run(mut self) -> EmResult<()> {
        let mut job_channel = self.service.listener().await?;
        self.load_jobs().await?;
        loop {
            let next_run = self
                .jobs
                .get(&self.next_job)
                .map(|next_run| {
                    let duration = next_run.timestamp_millis() - Utc::now().timestamp_millis();
                    StdDuration::from_millis(duration.clamp(0, i64::max_value()) as u64)
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
                    self.handle_action(notification?).await?
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
    async fn load_jobs(&mut self) -> EmResult<()> {
        info!("Requesting new job queue");
        let jobs = self.service.read_queued().await?;
        self.jobs.clear();
        self.next_job = jobs.get(0).map(|j| j.job_id).unwrap_or(0.into());
        for job in jobs {
            if let Some(duplicate) = self.jobs.get(&job.job_id) {
                return Err(EmError::DuplicateJobId(
                    job.job_id.into_inner(),
                    [job.next_run, duplicate.to_owned()],
                ));
            }
            self.jobs.insert(job.job_id, job.next_run);
        }
        Ok(())
    }

    /// Handle a piped [NotificationAction]. If the action is [NotificationAction::LoadJobs] then
    /// the jobs cache will be refreshed. If the action is [NotificationAction::CompleteJob] the
    /// the inner `job_id` will be used to mark a job as complete and jobs list will be refreshed.
    async fn handle_action(&mut self, action: NotificationAction) -> EmResult<()> {
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
    async fn run_next_job(&self) -> EmResult<()> {
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
    async fn complete_job(&self, job_id: &JobId) -> EmResult<()> {
        if !self.jobs.contains_key(job_id) {
            warn!(
                "Received a message to complete a job that is not in the job queue. Job_id = {}",
                job_id
            );
            return Ok(());
        };
        let job = self.service.read_one(job_id).await?;
        info!("Completing run for job_id = {}", job_id);
        let Err(error) = self.service.complete_job(job_id).await else {
            return Ok(())
        };
        self.send_error_email(job.maintainer(), format!("{error}"))
            .await?;
        Ok(())
    }

    /// Send an email to the specified `maintainer` with the error message as the email body
    async fn send_error_email(&self, maintainer: &str, message: String) -> EmResult<()> {
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
