use chrono::NaiveDateTime;
use common::{
    api::ApiRequestValidator,
    database::{
        connection::finalize_transaction,
        postgres::{listener::PgChangeListener, Postgres},
    },
    error::{EmError, EmResult},
};
use sqlx::{
    postgres::{types::PgInterval, PgListener},
    PgPool,
};

use crate::{
    job_worker::NotificationAction,
    services::{
        jobs::{
            Job, JobId, JobMin, JobRequest, JobRequestValidator, JobService, JobType, ScheduleEntry,
        },
        postgres::workflow_runs::PgWorkflowRunsService,
        workflow_runs::{WorkflowRun, WorkflowRunId, WorkflowRunStatus, WorkflowRunsService},
        workflows::WorkflowId,
    },
};

#[derive(Clone)]
pub struct PgJobsService {
    pool: PgPool,
    workflow_runs_service: PgWorkflowRunsService,
}

impl PgJobsService {
    /// Create a new interval job using the specified details from the parameters
    async fn create_interval_job(
        &self,
        workflow_id: &WorkflowId,
        maintainer: &str,
        interval: &PgInterval,
        next_run: &Option<NaiveDateTime>,
    ) -> EmResult<JobId> {
        let job_id = sqlx::query_scalar("select job.create_interval_job($1,$2,$3)")
            .bind(workflow_id)
            .bind(maintainer)
            .bind(interval)
            .bind(next_run)
            .fetch_one(&self.pool)
            .await?;
        Ok(job_id)
    }

    /// Create a new scheduled job using the specified details from the parameters
    async fn create_scheduled_job(
        &self,
        workflow_id: &WorkflowId,
        maintainer: &str,
        schedule: &[ScheduleEntry],
    ) -> EmResult<JobId> {
        let job_id = sqlx::query_scalar("select job.create_scheduled_job($1,$2,$3)")
            .bind(workflow_id)
            .bind(maintainer)
            .bind(schedule)
            .fetch_one(&self.pool)
            .await?;
        Ok(job_id)
    }
}

impl JobService for PgJobsService {
    type CreateRequestValidator = JobRequestValidator;
    type Database = Postgres;
    type Listener = PgChangeListener<NotificationAction>;
    type WorkflowRunService = PgWorkflowRunsService;

    fn create(pool: &PgPool, workflow_runs_service: &PgWorkflowRunsService) -> Self {
        Self {
            pool: pool.clone(),
            workflow_runs_service: workflow_runs_service.clone(),
        }
    }

    async fn create_job(&self, request: &JobRequest) -> EmResult<Job> {
        Self::CreateRequestValidator::validate(request)?;
        let JobRequest {
            workflow_id,
            maintainer,
            job_type,
            next_run,
        } = request;
        let job_id = match job_type {
            JobType::Scheduled(schedule) => {
                self.create_scheduled_job(workflow_id, maintainer, schedule)
                    .await?
            }
            JobType::Interval(interval) => {
                self.create_interval_job(workflow_id, maintainer, interval, next_run)
                    .await?
            }
        };
        self.read_one(&job_id).await
    }

    async fn read_one(&self, job_id: &JobId) -> EmResult<Job> {
        let job_option = sqlx::query_as(
            r#"
            select
                job_id, workflow_id, workflow_name, job_type, maintainer, job_schedule,
                job_interval, is_paused, next_run, current_workflow_run_id, workflow_run_status,
                progress, executor_id
            from job.v_jobs
            where job_id = $1"#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;
        job_option.map_or_else(
            || {
                Err(EmError::MissingRecord {
                    pk: job_id.to_string(),
                })
            },
            Ok,
        )
    }

    async fn read_many(&self) -> EmResult<Vec<Job>> {
        let result = sqlx::query_as(
            r#"
            select
                job_id, workflow_id, workflow_name, job_type, maintainer, job_schedule,
                job_interval, is_paused, next_run, current_workflow_run_id, workflow_run_status,
                progress, executor_id
            from job.v_jobs"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn read_queued(&self) -> EmResult<Vec<JobMin>> {
        let result = sqlx::query_as(
            r#"
            select job_id, next_run
            from job.v_queued_jobs"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn run_job(&self, job_id: &JobId) -> EmResult<Job> {
        let mut transaction = self.pool.begin().await?;
        let job_option: Option<(WorkflowId, bool)> = sqlx::query_as(
            r#"
                select j.workflow_id, j.is_paused
                from job.jobs j
                where j.job_id = $1
                for update
                skip locked"#,
        )
        .bind(job_id)
        .fetch_optional(&mut transaction)
        .await?;

        let workflow_id = match job_option {
            Some((_, is_paused)) if is_paused => {
                transaction.commit().await?;
                return Err(EmError::Generic(format!("Job, id = {job_id}, is paused")));
            }
            Some((workflow_id, _)) => workflow_id,
            None => {
                return Err(EmError::MissingRecord {
                    pk: job_id.to_string(),
                })
            }
        };

        let workflow_run_id = match self.workflow_runs_service.initialize(&workflow_id).await {
            Ok(WorkflowRun {
                workflow_run_id, ..
            }) => workflow_run_id,
            Err(error) => {
                transaction.rollback().await?;
                return Err(error);
            }
        };

        if let Err(error) = self.workflow_runs_service.schedule(&workflow_run_id).await {
            transaction.rollback().await?;
            return Err(error);
        };

        let query_result = sqlx::query("call job.set_job_as_running($1,$2)")
            .bind(job_id)
            .bind(workflow_run_id)
            .execute(&mut transaction)
            .await;

        finalize_transaction(query_result, transaction).await?;
        self.read_one(job_id).await
    }

    async fn complete_job(&self, job_id: &JobId) -> EmResult<Job> {
        let mut transaction = self.pool.begin().await?;
        let job_option: Option<Option<WorkflowRunId>> = sqlx::query_scalar(
            r#"
                select j.current_workflow_run_id
                from job.jobs j
                where j.job_id = $1
                for update
                skip locked"#,
        )
        .bind(job_id)
        .fetch_optional(&mut transaction)
        .await?;

        let workflow_run_id = match job_option {
            Some(Some(workflow_run_id)) => workflow_run_id,
            Some(None) => {
                transaction.commit().await?;
                return Err(EmError::Generic("Job must be active to finish".to_owned()));
            }
            None => {
                transaction.commit().await?;
                return Err(EmError::Generic(format!("No job for job_id = {job_id}")));
            }
        };

        let is_complete = match self.workflow_runs_service.read_one(&workflow_run_id).await {
            Ok(WorkflowRun { status, .. })
                if status == WorkflowRunStatus::Scheduled
                    || status == WorkflowRunStatus::Running =>
            {
                transaction.commit().await?;
                return Err(EmError::Generic(
                    "Workflow must be done to complete job".to_owned(),
                ));
            }
            Ok(WorkflowRun { status, .. }) => status == WorkflowRunStatus::Complete,
            Err(error) => {
                transaction.commit().await?;
                return Err(error);
            }
        };

        let query_result = sqlx::query("call job.complete_job($1,$2)")
            .bind(job_id)
            .bind(is_complete)
            .execute(&mut transaction)
            .await;

        finalize_transaction(query_result, transaction).await?;
        self.read_one(job_id).await
    }

    async fn listener(&self) -> EmResult<Self::Listener> {
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener.listen("jobs").await?;
        Ok(PgChangeListener::new(listener))
    }
}
