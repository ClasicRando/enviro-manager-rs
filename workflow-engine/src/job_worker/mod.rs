use std::collections::HashMap;

use chrono::Utc;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use log::{info, warn};
use sqlx::postgres::PgNotification;
use tokio::{
    signal::ctrl_c,
    time::{sleep as tokio_sleep, Duration as StdDuration},
};

use crate::{
    services::jobs::{Job, JobMin, JobsService},
    Error as WEError, Result as WEResult,
};

enum NotificationAction {
    LoadJobs,
    CompleteJob(i64),
}

pub struct JobWorker {
    service: &'static JobsService,
    jobs: HashMap<i64, JobMin>,
    next_job: i64,
}

impl JobWorker {
    pub async fn new(service: &'static JobsService) -> WEResult<Self> {
        Ok(Self {
            service,
            jobs: HashMap::new(),
            next_job: 0,
        })
    }

    pub async fn run(&mut self) -> WEResult<()> {
        let mut job_channel = self.service.listener().await?;
        self.load_jobs().await?;
        loop {
            let next_run = self
                .jobs
                .get(&self.next_job)
                .map(|j| {
                    let duration = j.next_run.timestamp_millis() - Utc::now().timestamp_millis();
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
                    self.handle_notification(notification).await?
                }
                _ = tokio_sleep(next_run) => {
                    self.run_next_job().await?;
                    self.load_jobs().await?;
                }
            }
        }
        Ok(())
    }

    async fn load_jobs(&mut self) -> WEResult<()> {
        info!("Requesting new job queue");
        let jobs = self.service.read_queued().await?;
        self.jobs.clear();
        self.next_job = jobs.get(0).map(|j| j.job_id).unwrap_or(0);
        for job in jobs {
            if let Some(duplicate) = self.jobs.get(&job.job_id) {
                return Err(WEError::DuplicateJobId(
                    job.job_id,
                    [job.next_run, duplicate.next_run.to_owned()],
                ));
            }
            self.jobs.insert(job.job_id, job);
        }
        Ok(())
    }

    fn parse_notification(
        &mut self,
        result: Result<PgNotification, sqlx::Error>,
    ) -> WEResult<NotificationAction> {
        let notifcation = result?;
        let payload = notifcation.payload();
        if payload.is_empty() {
            return Ok(NotificationAction::LoadJobs);
        }
        let Ok(job_id) = payload.parse::<i64>() else {
            return Err(WEError::PayloadParseError(payload.to_owned()))
        };
        info!("Received notifcation of \"{}\"", payload);
        Ok(NotificationAction::CompleteJob(job_id))
    }

    async fn handle_notification(
        &mut self,
        notification: Result<PgNotification, sqlx::Error>,
    ) -> WEResult<()> {
        match self.parse_notification(notification) {
            Ok(action) => match action {
                NotificationAction::LoadJobs => self.load_jobs().await?,
                NotificationAction::CompleteJob(job_id) => {
                    self.complete_job(job_id).await?;
                    self.load_jobs().await?;
                }
            },
            Err(error) => return Err(error),
        }
        Ok(())
    }

    async fn run_next_job(&self) -> WEResult<()> {
        let Some(job) = self.jobs.get(&self.next_job) else {
            warn!("Attempted to run a job that is not in the job queue. Job_id = {}", self.next_job);
            return Ok(())
        };
        info!("Starting new job run for job_id = {}", job.job_id);
        if job.next_run > Utc::now().naive_utc() {
            return Err(WEError::JobNotReady);
        }
        self.service.run_job(job.job_id).await?;
        Ok(())
    }

    async fn complete_job(&self, job_id: i64) -> WEResult<()> {
        let Some(job) = self.jobs.get(&job_id) else {
            warn!("Received a message to complete a job that is not in the job queue. Job_id = {}", job_id);
            return Ok(())
        };
        let Some(Job { maintainer, .. }) = self.service.read_one(job_id).await? else {
            return Err(WEError::Generic(format!("Could not find a job in the database for job_id = {}", job_id)))
        };
        info!("Completing run for job_id = {}", job.job_id);
        let Err(WEError::Generic(error)) = self.service.complete_job(job.job_id).await else {
            return Ok(())
        };
        self.send_error_email(maintainer, error).await?;
        Ok(())
    }

    async fn send_error_email(&self, maintainer: String, message: String) -> WEResult<()> {
        warn!(
            "Sending error email to {} with message\n{}",
            maintainer, message
        );
        let username = env!("CLIPPY_USERNAME");
        let email = Message::builder()
            .from(format!("Clippy <{}>", username).parse()?)
            .to(maintainer.parse()?)
            .subject("Job Completion Error")
            .body(message)?;
        let credentials = Credentials::from((username, env!("CLIPPY_PASSWORD")));
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(env!("CLIPPY_RELAY"))?
            .credentials(credentials)
            .build();

        let _response = mailer.send(email).await?;
        Ok(())
    }
}
