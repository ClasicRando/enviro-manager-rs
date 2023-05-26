use chrono::NaiveDateTime;
use common::error::EmResult;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use sqlx::{postgres::types::PgInterval, Database, Pool};

use super::workflow_runs::WorkflowRunStatus;
use crate::{
    database::listener::ChangeListener, job_worker::NotificationAction, WorkflowRunsService,
};

/// Represents the `job_type` Postgresql enum value within the database. Should never be used by
/// itself but rather used to parse into the [JobType] enum that hold the job running details.
#[derive(sqlx::Type)]
#[sqlx(type_name = "job_type")]
pub enum JobTypeEnum {
    Scheduled,
    Interval,
}

/// Details of a [JobType::Scheduled] job. Specifies a single run of the job as a `day_of_the_week`
/// (Monday = 1, Sunday = 7) and a time within the day (timestamp without a timezone). Links to a
/// postgresql composite type.
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

/// Minimum details about a job to execute. Details fetched from `job.v_queued_jobs` and later
/// packed into a hashmap (key = `job_id`). The `next_run` value is the next time the job needs to
/// be executed.
#[derive(sqlx::FromRow)]
pub struct JobMin {
    pub job_id: i64,
    pub next_run: NaiveDateTime,
}

const PG_INTERVAL_FIELDS: &[&str] = &["months", "days", "years"];

/// Deserialization method for [PgInterval] to convert from a serialized object containing a
/// months, days and microseconds value. This allows for [PgInterval] to be extracted from a
/// [JobType::Interval] value serialized within a [JobRequest].
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

/// Serialization method for [PgInterval] to convert from a serialized object containing a months,
/// days and microseconds value. This allows for [PgInterval] to be serialized into a
/// [JobType::Scheduled] value within a [JobRequest].
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

/// Describes the only difference between job entry types. Jobs are either scheduled with a 1 or
/// more weekly schedule entries or follow an interval schedule of a defined period between runs.
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

/// Job details as fetched from `job.v_jobs`. Contains the job and underlining workflow details as
/// well as the current workflow run (if any) for the job.
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

/// API request data for updating a job entry. Specifies all fields within the record except for
/// the `job_id` which should be provided by a path parameter
#[derive(Deserialize)]
pub struct JobRequest {
    pub(crate) workflow_id: i64,
    pub(crate) maintainer: String,
    pub(crate) job_type: JobType,
    pub(crate) next_run: Option<NaiveDateTime>,
}

/// Wrapper for a `job_id` value. Made to ensure data passed as the id of a job is correct and not
/// just any i64 value.
#[derive(sqlx::Type, Eq, Hash, PartialEq, Deserialize)]
#[sqlx(transparent)]
pub struct JobId(i64);

impl From<i64> for JobId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Service for fetching and interacting with task data. Wraps a [PgPool] and provides
/// interaction methods for the API and [JobWorker][crate::job_worker::JobWorker].
pub trait JobService: Clone + Send {
    type Database: Database;
    type Listener: ChangeListener<NotificationAction>;
    type WorkflowRunService: WorkflowRunsService<Database = Self::Database>;

    /// Create a new [JobService] with the referenced pool as the data source
    fn create(
        pool: &Pool<Self::Database>,
        workflow_runs_service: &Self::WorkflowRunService,
    ) -> Self;
    /// Create a new job with the data contained within `request`. Branches to specific calls for
    /// [JobType::Scheduled] and [JobType::Interval].
    async fn create_job(&self, request: JobRequest) -> EmResult<Job>;
    /// Read a single job record from `job.v_jobs` for the specified `job_id`. Will return [None]
    /// when the id does not match a record
    async fn read_one(&self, job_id: &JobId) -> EmResult<Job>;

    /// Read all job records found from `job.v_jobs`
    async fn read_many(&self) -> EmResult<Vec<Job>>;
    /// Read all job records from `job.v_queued_jobs`. This excludes all job entries that are
    /// paused or currently have a workflow run that not complete. Ordered by the `next_run` field
    async fn read_queued(&self) -> EmResult<Vec<JobMin>>;
    /// Run the job specified by the `job_id`. Returns the [Job] entry if the `job_id` matches a
    /// record
    async fn run_job(&self, job_id: &JobId) -> EmResult<Job>;
    /// Complete the job specified by the `job_id`. Returns the [Job] entry if the `job_id` matches
    /// a record
    async fn complete_job(&self, job_id: &JobId) -> EmResult<Job>;
    /// Get a [ChangeListener] for updates on the job queue this service is watching.
    async fn listener(&self) -> EmResult<Self::Listener>;
}
