use std::{str::FromStr, string::FromUtf8Error};

use actix_session::Session;
use actix_web::{web, HttpResponse};
use chrono::{NaiveDateTime, NaiveTime};
use common::api::ApiResponseBody;
use leptos::*;
use reqwest::Method;
use serde::Deserialize;
use thiserror::Error;
use workflow_engine::{
    job::data::{Job, JobId, JobRequest, JobType, JobTypeEnum, ScheduleEntry},
    workflow::data::WorkflowId,
};

use crate::{
    api::workflow_engine::workflows::get_workflows,
    components::workflow_engine::main_page::{
        JobScheduleEntry, Jobs, JobsTab, NewIntervalJob, NewJobModal, NewJobNextRun,
        NewScheduledJob,
    },
    error_if, extract_session_uid, utils,
    utils::HtmxResponseBuilder,
    ServerFnError,
};

pub fn service() -> actix_web::Scope {
    web::scope("/jobs")
        .service(
            web::resource("")
                .route(web::get().to(jobs))
                .route(web::post().to(create_job)),
        )
        .route("/create-modal", web::post().to(create_job_modal))
        .route("/tab", web::get().to(jobs_tab))
        .route("/next-run", web::get().to(next_run_input))
        .route("/job-type", web::get().to(job_type_container))
        .route("/job-schedule-entry", web::get().to(job_schedule_entry))
}

async fn jobs_html_with_extras(
    session: Session,
    is_tab: bool,
    modal_id: Option<String>,
    toast_message: Option<String>,
) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }
    let jobs = match get_jobs().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };

    let mut builder = HtmxResponseBuilder::new();
    if let Some(modal_id) = modal_id {
        builder.add_close_modal_event(modal_id);
    }
    if let Some(message) = toast_message {
        builder.add_create_toast_event(message);
    }
    builder.html_chunk(move |cx| {
        if is_tab {
            view! { cx, <JobsTab jobs=jobs/> }.into_view(cx)
        } else {
            view! { cx, <Jobs jobs=jobs/> }.into_view(cx)
        }
    })
}

async fn jobs_html(session: Session, is_tab: bool) -> HttpResponse {
    jobs_html_with_extras(session, is_tab, None, None).await
}

async fn jobs(session: Session) -> HttpResponse {
    jobs_html(session, false).await
}

async fn jobs_tab(session: Session) -> HttpResponse {
    jobs_html(session, true).await
}

async fn get_jobs() -> Result<Vec<Job>, ServerFnError> {
    let jobs_response = utils::api_request(
        "http://127.0.0.1:8000/api/v1/jobs?f=msgpack",
        Method::GET,
        None::<String>,
        None::<()>,
    )
    .await?;
    let jobs = match jobs_response {
        ApiResponseBody::Success(inner) => inner,
        ApiResponseBody::Message(message) => {
            return utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            return utils::server_fn_error!(message)
        }
    };
    Ok(jobs)
}

async fn create_job_modal(session: Session) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }

    let workflows = match get_workflows().await {
        Ok(inner) => inner,
        Err(error) => return error.to_response(),
    };

    HtmxResponseBuilder::new().html_chunk(|cx| {
        view! { cx, <NewJobModal workflows=workflows/> }
    })
}

#[derive(Deserialize)]
struct NextRun {
    #[serde(default)]
    next_run_chk: Option<String>,
}

async fn next_run_input(next_run: web::Query<NextRun>) -> HttpResponse {
    if next_run.next_run_chk.is_some() {
        return HtmxResponseBuilder::new().html_chunk(|cx| {
            view! { cx, <NewJobNextRun/> }
        });
    }
    HtmxResponseBuilder::new().static_body("")
}

#[derive(Deserialize)]
struct JobTypeQuery {
    job_type: JobTypeEnum,
}

async fn job_type_container(query: web::Query<JobTypeQuery>) -> HttpResponse {
    HtmxResponseBuilder::new().html_chunk(move |cx| match query.job_type {
        JobTypeEnum::Scheduled => view! { cx, <NewScheduledJob/> },
        JobTypeEnum::Interval => view! { cx, <NewIntervalJob/> },
    })
}

async fn job_schedule_entry() -> HttpResponse {
    HtmxResponseBuilder::new().html_chunk(|cx| {
        view! { cx, <JobScheduleEntry/> }
    })
}

#[derive(Error, Debug)]
enum CreateJobBuilderError {
    #[error("Missing value for field: {0}")]
    MissingField(&'static str),
    #[error(
        "Incorrect type for field `{field}`. Found value of `{value}` that could not be converted \
         to `{expected_type}`"
    )]
    IncorrectType {
        field: &'static str,
        value: String,
        expected_type: &'static str,
    },
    #[error("Schedule attributes must come in pairs but found mismatched vector lengths")]
    ScheduleAttributesLengthMismatch,
    #[error("Schedule cannot be empty")]
    EmptySchedule,
    #[error("Cannot provide an interval with negative components")]
    NegativeIntervalValue,
    #[error("Cannot provide an interval of zero")]
    ZeroInterval,
    #[error("Could not deserialize payload, `{payload}`. Error: {error}")]
    Deserialize {
        payload: String,
        error: FromUtf8Error,
    },
    #[error("Could not parse payload, `{0}`")]
    Parse(String),
    #[error("{0}")]
    RuleBroken(String),
}

impl From<(&'static str, &str, &'static str)> for CreateJobBuilderError {
    fn from(value: (&'static str, &str, &'static str)) -> Self {
        Self::IncorrectType {
            field: value.0,
            value: value.1.to_owned(),
            expected_type: value.2,
        }
    }
}

#[derive(Default)]
struct CreateJobBuilder {
    workflow_id: Option<WorkflowId>,
    maintainer: Option<String>,
    job_type: Option<JobTypeEnum>,
    next_run: Option<NaiveDateTime>,
    day_of_the_week: Vec<i16>,
    time_of_day: Vec<NaiveTime>,
    months: Option<i32>,
    days: Option<i32>,
    minutes: Option<i32>,
    modal_id: Option<String>,
}

impl CreateJobBuilder {
    fn workflow_id(&mut self, workflow_id: WorkflowId) {
        if self.workflow_id.is_some() {
            log::warn!("Found duplicate workflow_id in create_job request, `{workflow_id}`");
            return;
        }
        self.workflow_id = Some(workflow_id);
    }

    fn maintainer(&mut self, maintainer: String) {
        if self.maintainer.is_some() {
            log::warn!("Found duplicate maintainer in create_job request, `{maintainer}`");
            return;
        }
        self.maintainer = Some(maintainer);
    }

    fn job_type(&mut self, job_type: JobTypeEnum) {
        if self.job_type.is_some() {
            log::warn!("Found duplicate job_type in create_job request, `{job_type:?}`");
            return;
        }
        self.job_type = Some(job_type);
    }

    fn next_run(&mut self, next_run: NaiveDateTime) {
        if self.next_run.is_some() {
            log::warn!("Found duplicate next_run in create_job request, `{next_run}`");
            return;
        }
        self.next_run = Some(next_run);
    }

    fn day_of_the_week(&mut self, day_of_the_week: i16) {
        self.day_of_the_week.push(day_of_the_week);
    }

    fn time_of_day(&mut self, time_of_day: NaiveTime) {
        self.time_of_day.push(time_of_day);
    }

    fn months(&mut self, months: i32) {
        if self.months.is_some() {
            log::warn!("Found duplicate months in create_job request, `{months}`");
            return;
        }
        self.months = Some(months);
    }

    fn days(&mut self, days: i32) {
        if self.days.is_some() {
            log::warn!("Found duplicate days in create_job request, `{days}`");
            return;
        }
        self.days = Some(days);
    }

    fn minutes(&mut self, minutes: i32) {
        if self.minutes.is_some() {
            log::warn!("Found duplicate minutes in create_job request, `{minutes}`");
            return;
        }
        self.minutes = Some(minutes);
    }

    fn modal_id(&mut self, modal_id: String) {
        if self.modal_id.is_some() {
            log::warn!("Found duplicate modal_id in create_job request, `{modal_id}`");
            return;
        }
        self.modal_id = Some(modal_id);
    }

    fn build(self) -> Result<CreateJob, CreateJobBuilderError> {
        let job_type_enum = self
            .job_type
            .ok_or(CreateJobBuilderError::MissingField("job_type"))?;
        let job_type = match job_type_enum {
            JobTypeEnum::Scheduled => {
                if self.day_of_the_week.len() != self.time_of_day.len() {
                    return Err(CreateJobBuilderError::ScheduleAttributesLengthMismatch);
                }
                if self.day_of_the_week.is_empty() {
                    return Err(CreateJobBuilderError::EmptySchedule);
                }
                let entries = self
                    .day_of_the_week
                    .into_iter()
                    .zip(self.time_of_day)
                    .map(|(dow, tod)| ScheduleEntry::new(dow, tod))
                    .collect();
                JobType::new_scheduled(entries)
            }
            JobTypeEnum::Interval => {
                let months = self
                    .months
                    .ok_or(CreateJobBuilderError::MissingField("months"))?;
                let days = self
                    .days
                    .ok_or(CreateJobBuilderError::MissingField("days"))?;
                let minutes = self
                    .minutes
                    .map(|m| (m as i64) * 60 * 1000 * 1000)
                    .ok_or(CreateJobBuilderError::MissingField("minutes"))?;
                if months < 0 || days < 0 || minutes < 0 {
                    return Err(CreateJobBuilderError::NegativeIntervalValue);
                }
                if months == 0 && days == 0 && minutes == 0 {
                    return Err(CreateJobBuilderError::ZeroInterval);
                }
                JobType::new_interval(months, days, minutes)
            }
        };
        let maintainer = self
            .maintainer
            .ok_or(CreateJobBuilderError::MissingField("maintainer"))?;
        Ok(CreateJob {
            workflow_id: self
                .workflow_id
                .ok_or(CreateJobBuilderError::MissingField("workflow_id"))?,
            maintainer: error_if(
                maintainer,
                |v| v.is_empty(),
                |_| CreateJobBuilderError::RuleBroken("Maintainer cannot be empty".to_owned()),
            )?,
            next_run: self.next_run,
            job_type,
            modal_id: self
                .modal_id
                .ok_or(CreateJobBuilderError::MissingField("modal_id"))?,
        })
    }
}

#[derive(Debug)]
struct CreateJob {
    workflow_id: WorkflowId,
    maintainer: String,
    next_run: Option<NaiveDateTime>,
    job_type: JobType,
    modal_id: String,
}

impl FromStr for CreateJob {
    type Err = CreateJobBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = CreateJobBuilder::default();
        let decoded = urlencoding::decode(s).map_err(|e| CreateJobBuilderError::Deserialize {
            payload: s.to_owned(),
            error: e,
        })?;
        for pair in decoded.split('&').map(|s| s.split_once('=')) {
            let Some((key, value)) = pair else {
                return Err(CreateJobBuilderError::Parse(s.to_owned()));
            };
            match key {
                "workflow_id" => {
                    let workflow_id = value
                        .parse::<i64>()
                        .map_err(|_| ("workflow_id", value, "WorkflowId"))?;
                    builder.workflow_id(workflow_id.into());
                }
                "maintainer" => builder.maintainer(value.to_owned()),
                "job_type" => builder.job_type(
                    value
                        .parse()
                        .map_err(|_| ("job_type", value, "JobTypeEnum"))?,
                ),
                "next_run" => {
                    builder.next_run(value.parse().map_err(|_| ("next_run", value, "DateTime"))?)
                }
                "day_of_the_week" => builder.day_of_the_week(
                    value
                        .parse()
                        .map_err(|_| ("day_of_the_week", value, "i16"))?,
                ),
                "time_of_day" => builder.time_of_day(
                    format!("{value}:00")
                        .parse()
                        .map_err(|_| ("time_of_day", value, "Time"))?,
                ),
                "months" => {
                    if value.is_empty() {
                        builder.months(0);
                        continue;
                    }
                    builder.months(value.parse().map_err(|_| ("months", value, "i32"))?)
                }
                "days" => {
                    if value.is_empty() {
                        builder.days(0);
                        continue;
                    }
                    builder.days(value.parse().map_err(|_| ("days", value, "i32"))?)
                }
                "minutes" => {
                    if value.is_empty() {
                        builder.minutes(0);
                        continue;
                    }
                    builder.minutes(value.parse().map_err(|_| ("minutes", value, "i32"))?)
                }
                "modal_id" => builder.modal_id(value.to_owned()),
                _ => continue,
            }
        }
        builder.build()
    }
}

async fn create_job(session: Session, payload: String) -> HttpResponse {
    if extract_session_uid(&session).is_err() {
        return HtmxResponseBuilder::location_login();
    }

    let CreateJob {
        workflow_id,
        maintainer,
        next_run,
        job_type,
        modal_id,
    } = match CreateJob::from_str(&payload) {
        Ok(inner) => inner,
        Err(error) => return HtmxResponseBuilder::modal_error_message(error.to_string()),
    };

    let job_request = JobRequest::new(workflow_id, maintainer, job_type, next_run);
    let toast_message = match post_create_job(job_request).await {
        Ok(job_id) => format!("Created new job, ID: {job_id}"),
        Err(error) => return error.to_response(),
    };

    jobs_html_with_extras(session, false, Some(modal_id), Some(toast_message)).await
}

async fn post_create_job(job_request: JobRequest) -> Result<JobId, ServerFnError> {
    let clean_executors_response: ApiResponseBody<Job> = utils::api_request(
        "http://127.0.0.1:8000/api/v1/jobs?f=msgpack",
        Method::POST,
        None::<String>,
        Some(job_request),
    )
    .await?;
    match clean_executors_response {
        ApiResponseBody::Success(job) => {
            log::info!("Create new job: {}", job.job_id);
            Ok(job.job_id)
        }
        ApiResponseBody::Message(message) => {
            log::info!("{message}");
            utils::server_fn_error!("Expected data, got message")
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}
