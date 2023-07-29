use std::str::FromStr;

use chrono::{NaiveDateTime, NaiveTime};
use common::{api::ApiRequestValidator, error::EmError};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use sqlx::postgres::types::PgInterval;

use crate::{
    executor::data::ExecutorId,
    workflow::data::WorkflowId,
    workflow_run::data::{WorkflowRunId, WorkflowRunStatus},
};

/// Represents the `job_type` Postgresql enum value within the database. Should never be used by
/// itself but rather used to parse into the [JobType] enum that hold the job running details.
#[derive(sqlx::Type, Deserialize, Debug)]
#[sqlx(type_name = "job_type")]
pub enum JobTypeEnum {
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "interval")]
    Interval,
}

impl FromStr for JobTypeEnum {
    type Err = EmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scheduled" => Ok(Self::Scheduled),
            "interval" => Ok(Self::Interval),
            _ => Err(EmError::Generic(format!(
                "Parse JobTypeEnum from string. Expected `scheduled` or `interval` but got `{s}`"
            ))),
        }
    }
}

/// Details of a [JobType::Scheduled] job. Specifies a single run of the job as a `day_of_the_week`
/// (Monday = 1, Sunday = 7) and a time within the day (timestamp without a timezone). Links to a
/// postgresql composite type.
#[derive(sqlx::Type, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[sqlx(type_name = "schedule_entry")]
pub struct ScheduleEntry {
    day_of_the_week: i16,
    time_of_day: NaiveTime,
}

impl ScheduleEntry {
    /// Create a new [ScheduleEntry]
    pub const fn new(day_of_the_week: i16, time_of_day: NaiveTime) -> Self {
        Self {
            day_of_the_week,
            time_of_day,
        }
    }
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
    pub job_id: JobId,
    pub next_run: NaiveDateTime,
}

const PG_INTERVAL_FIELDS: &[&str] = &["months", "days", "years"];

/// Deserialization method for [PgInterval] to convert from a serialized object containing a
/// months, days and microseconds value. This allows for [PgInterval] to be extracted from a
/// [JobType::Interval] value serialized within a [JobRequest].
#[allow(clippy::indexing_slicing)]
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
                days: days.ok_or_else(|| serde::de::Error::missing_field(PG_INTERVAL_FIELDS[1]))?,
                microseconds: microseconds
                    .ok_or_else(|| serde::de::Error::missing_field(PG_INTERVAL_FIELDS[2]))?,
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
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize)]
pub struct Job {
    pub job_id: JobId,
    pub workflow_id: WorkflowId,
    pub workflow_name: String,
    pub job_type: JobType,
    pub maintainer: String,
    pub is_paused: bool,
    pub next_run: NaiveDateTime,
    pub current_workflow_run_id: Option<WorkflowRunId>,
    pub workflow_run_status: Option<WorkflowRunStatus>,
    pub executor_id: Option<ExecutorId>,
    pub progress: Option<i16>,
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
#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequest {
    /// ID of the workflow that is to be executed as the [Job]
    pub(crate) workflow_id: WorkflowId,
    /// Email address of the maintainer to be sent a message if the job fails
    pub(crate) maintainer: String,
    /// Type of job that is be created. Contains the details of how the job is to be executed
    pub(crate) job_type: JobType,
    /// Optional datetime that defines when the next run of the job is to be executed. If [None]
    /// then the system will calculate when the next run should be.
    pub(crate) next_run: Option<NaiveDateTime>,
}

impl JobRequest {
    pub const fn new(
        workflow_id: WorkflowId,
        maintainer: String,
        job_type: JobType,
        next_run: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            workflow_id,
            maintainer,
            job_type,
            next_run,
        }
    }
}

/// API request validator for [JobRequest]
pub struct JobRequestValidator;

impl ApiRequestValidator for JobRequestValidator {
    type ErrorMessage = &'static str;
    type Request = JobRequest;

    fn validate(request: &Self::Request) -> Result<(), Self::ErrorMessage> {
        if request.maintainer.trim().is_empty() {
            return Err("Maintainer must not be empty or whitespace");
        }

        let JobType::Scheduled(entries) = &request.job_type else {
            return Ok(());
        };

        let mut seen = std::collections::HashSet::new();
        for entry in entries {
            if entry.day_of_the_week > 7 || entry.day_of_the_week < 1 {
                return Err(
                    "All schedule entries must have a 'day_of_the_week' attribute between 1 and 7",
                );
            }
            if !seen.insert(entry) {
                return Err("Schedule Entry objects must not duplicate for a single job");
            }
        }

        Ok(())
    }
}

/// Wrapper for a `job_id` value. Made to ensure data passed as the id of a job is correct and not
/// just any i64 value.
#[derive(sqlx::Type, Eq, Hash, PartialEq, Deserialize, Serialize, Clone, Copy)]
#[sqlx(transparent)]
pub struct JobId(i64);

impl From<i64> for JobId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl JobId {
    /// Extract the inner [`i64`] value
    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
