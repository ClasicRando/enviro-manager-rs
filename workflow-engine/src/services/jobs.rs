use chrono::NaiveDateTime;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use sqlx::{postgres::{types::PgInterval, PgListener}, PgPool};

use crate::{
    database::finish_transaction,
    error::{Error as WEError, Result as WEResult},
};

use super::workflow_runs::WorkflowRunStatus;

#[derive(sqlx::Type)]
#[sqlx(type_name = "job_type")]
pub enum JobTypeEnum {
    Scheduled,
    Interval,
}

#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "schedule_entry")]
pub struct ScheduleEntry {
    day_of_the_week: i16,
    time_of_day: NaiveDateTime,
}

impl sqlx::postgres::PgHasArrayType for ScheduleEntry {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_schedule_entry")
    }
}

#[derive(sqlx::FromRow)]
pub struct JobMin {
    pub job_id: i64,
    pub next_run: NaiveDateTime,
}

const PG_INTERVAL_FIELDS: &[&str] = &["months", "days", "years"];

fn deserialize_interval<'de, D>(deserializer: D) -> Result<PgInterval, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(field_identifier, rename_all = "lowercase")]
    enum Field {
        Months,
        Days,
        Microseconds,
    }

    struct PgIntervalVisitor;

    impl<'de> Visitor<'de> for PgIntervalVisitor {
        type Value = PgInterval;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("struct PgInterval")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut months = None;
            let mut days = None;
            let mut microseconds = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::Months => {
                        if months.is_some() {
                            return Err(serde::de::Error::duplicate_field(PG_INTERVAL_FIELDS[0]));
                        }
                        months = Some(map.next_value()?)
                    }
                    Field::Days => {
                        if days.is_some() {
                            return Err(serde::de::Error::duplicate_field(PG_INTERVAL_FIELDS[1]));
                        }
                        days = Some(map.next_value()?)
                    }
                    Field::Microseconds => {
                        if microseconds.is_some() {
                            return Err(serde::de::Error::duplicate_field(PG_INTERVAL_FIELDS[2]));
                        }
                        microseconds = Some(map.next_value()?)
                    }
                }
            }
            Ok(PgInterval {
                months: months
                    .ok_or_else(|| serde::de::Error::missing_field(PG_INTERVAL_FIELDS[0]))?,
                days: days.ok_or_else(|| serde::de::Error::missing_field(PG_INTERVAL_FIELDS[0]))?,
                microseconds: microseconds
                    .ok_or_else(|| serde::de::Error::missing_field(PG_INTERVAL_FIELDS[0]))?,
            })
        }
    }

    deserializer.deserialize_struct("PgInterval", PG_INTERVAL_FIELDS, PgIntervalVisitor)
}

fn serialize_interval<S>(interval: &PgInterval, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut pg_interval = serializer.serialize_struct("PgInterval", 3)?;
    pg_interval.serialize_field("months", &interval.months)?;
    pg_interval.serialize_field("days", &interval.days)?;
    pg_interval.serialize_field("microseconds", &interval.microseconds)?;
    pg_interval.end()
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JobType {
    Scheduled(Vec<ScheduleEntry>),
    #[serde(
        serialize_with = "serialize_interval",
        deserialize_with = "deserialize_interval"
    )]
    Interval(PgInterval),
}

#[derive(Serialize)]
pub struct Job {
    job_id: i64,
    workflow_id: i64,
    workflow_name: String,
    job_type: JobType,
    pub maintainer: String,
    is_paused: bool,
    next_run: NaiveDateTime,
    current_workflow_run_id: i64,
    workflow_run_status: Option<WorkflowRunStatus>,
    executor_id: Option<i64>,
    progress: i16,
}

impl<'r, R> sqlx::FromRow<'r, R> for Job
where
    R: sqlx::Row,
    &'r str: sqlx::ColumnIndex<R>,
    &'r str: sqlx::Decode<'r, R::Database>,
    &'r str: sqlx::Type<R::Database>,
    String: sqlx::Decode<'r, R::Database>,
    String: sqlx::Type<R::Database>,
    i64: sqlx::Decode<'r, R::Database>,
    i64: sqlx::Type<R::Database>,
    i16: sqlx::Decode<'r, R::Database>,
    i16: sqlx::Type<R::Database>,
    bool: sqlx::Decode<'r, R::Database>,
    bool: sqlx::Type<R::Database>,
    Vec<ScheduleEntry>: sqlx::Decode<'r, R::Database>,
    Vec<ScheduleEntry>: sqlx::Type<R::Database>,
    PgInterval: sqlx::Decode<'r, R::Database>,
    PgInterval: sqlx::Type<R::Database>,
    NaiveDateTime: sqlx::Decode<'r, R::Database>,
    NaiveDateTime: sqlx::Type<R::Database>,
    WorkflowRunStatus: sqlx::Decode<'r, R::Database>,
    WorkflowRunStatus: sqlx::Type<R::Database>,
    JobTypeEnum: sqlx::Decode<'r, R::Database>,
    JobTypeEnum: sqlx::Type<R::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
        let cron_job_type: JobTypeEnum = row.try_get("job_type")?;
        let job_type = match cron_job_type {
            JobTypeEnum::Scheduled => JobType::Scheduled(row.try_get("job_schedule")?),
            JobTypeEnum::Interval => JobType::Interval(row.try_get("job_interval")?),
        };
        Ok(Self {
            job_id: row.try_get("job_id")?,
            workflow_id: row.try_get("workflow_id")?,
            workflow_name: row.try_get("workflow_name")?,
            job_type,
            maintainer: row.try_get("maintainer")?,
            is_paused: row.try_get("is_paused")?,
            next_run: row.try_get("next_run")?,
            current_workflow_run_id: row.try_get("current_workflow_run_id")?,
            workflow_run_status: row.try_get("workflow_run_status")?,
            executor_id: row.try_get("executor_id")?,
            progress: row.try_get("progress")?,
        })
    }
}

#[derive(Deserialize)]
pub struct JobRequest {
    workflow_id: i64,
    maintainer: String,
    job_type: JobType,
    next_run: Option<NaiveDateTime>,
}

pub struct JobsService {
    pool: &'static PgPool,
}

impl JobsService {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: JobRequest) -> WEResult<Job> {
        let job_id = match &request.job_type {
            JobType::Scheduled(schedule) => {
                self.create_scheduled_job(&request.workflow_id, &request.maintainer, schedule)
                    .await?
            }
            JobType::Interval(interval) => {
                self.create_interval_job(
                    &request.workflow_id,
                    &request.maintainer,
                    interval,
                    &request.next_run,
                )
                .await?
            }
        };
        match self.read_one(job_id).await {
            Ok(Some(job)) => Ok(job),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn create_interval_job(
        &self,
        workflow_id: &i64,
        maintainer: &str,
        interval: &PgInterval,
        next_run: &Option<NaiveDateTime>,
    ) -> WEResult<i64> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query_scalar("select create_interval_cron_job($1,$2,$3)")
            .bind(workflow_id)
            .bind(maintainer)
            .bind(interval)
            .bind(next_run)
            .fetch_one(&mut transaction)
            .await;
        let job_id: i64 = finish_transaction(transaction, result).await?;
        Ok(job_id)
    }

    async fn create_scheduled_job(
        &self,
        workflow_id: &i64,
        maintainer: &str,
        schedule: &[ScheduleEntry],
    ) -> WEResult<i64> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query_scalar("select create_scheduled_cron_job($1,$2,$3)")
            .bind(workflow_id)
            .bind(maintainer)
            .bind(schedule)
            .fetch_one(&mut transaction)
            .await;
        let job_id: i64 = finish_transaction(transaction, result).await?;
        Ok(job_id)
    }

    pub async fn read_one(&self, job_id: i64) -> WEResult<Option<Job>> {
        let result = sqlx::query_as(
            r#"
            select job_id, workflow_id, workflow_name, job_type, maintainer, job_schedule, job_interval, is_paused,
                   next_run, current_workflow_run_id, workflow_run_status, progress, executor_id
            from   v_jobs
            where  job_id = $1"#,
        )
        .bind(job_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> WEResult<Vec<Job>> {
        let result = sqlx::query_as(
            r#"
            select job_id, workflow_id, workflow_name, job_type, maintainer, job_schedule, job_interval, is_paused,
                   next_run, current_workflow_run_id, workflow_run_status, progress, executor_id
            from   v_jobs"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_queued(&self) -> WEResult<Vec<JobMin>> {
        let result = sqlx::query_as(
            r#"
            select job_id, next_run
            from   v_queued_jobs"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(result)
    }

    pub async fn run_job(&self, job_id: i64) -> WEResult<Option<Job>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query("call run_job($1)")
            .bind(job_id)
            .execute(&mut transaction)
            .await;
        finish_transaction(transaction, result).await?;
        self.read_one(job_id).await
    }

    pub async fn complete_job(&self, job_id: i64) -> WEResult<Option<Job>> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query_scalar("select complete_job($1)")
            .bind(job_id)
            .fetch_one(&mut transaction)
            .await;
        let message: String = match result {
            Ok(inner) => inner,
            Err(error) => {
                transaction.rollback().await?;
                return Err(error.into());
            }
        };
        if message.is_empty() {
            transaction.commit().await?;
            return self.read_one(job_id).await;
        }
        transaction.rollback().await?;
        Err(WEError::Generic(message))
    }

    pub async fn listener(&self) -> WEResult<PgListener> {
        let mut listener = PgListener::connect_with(self.pool).await?;
        listener
            .listen("jobs")
            .await?;
        Ok(listener)
    }
}
