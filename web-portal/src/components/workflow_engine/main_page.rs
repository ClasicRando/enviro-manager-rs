use chrono::NaiveDateTime;
use leptos::*;
use strum::{EnumIter, IntoEnumIterator};
use workflow_engine::{
    executor::data::{Executor, ExecutorId},
    job::data::{Job, JobId, JobType, ScheduleEntry},
    workflow::data::{Workflow, WorkflowId},
    workflow_run::data::{WorkflowRun, WorkflowRunId, WorkflowRunStatus, WorkflowRunTask},
};

use crate::components::{
    grid::{Col, Row},
    into_view, into_view_option,
    modal::{CreateModal, ADD_MODAL_SWAP, ADD_MODAL_TARGET},
    table::{DataTableExtras, ExtraTableButton, RowAction, RowWithDetails},
};

#[component]
pub fn WorkflowRunTask(cx: Scope, workflow_run_task: WorkflowRunTask) -> impl IntoView {
    view! { cx,
        <tr>
            <td>{into_view(workflow_run_task.task_order)}</td>
            <td>{into_view(workflow_run_task.task_id)}</td>
            <td>{into_view(workflow_run_task.name)}</td>
            <td>{into_view(workflow_run_task.description)}</td>
            <td>{into_view(workflow_run_task.task_status)}</td>
            <td>{into_view_option(workflow_run_task.parameters)}</td>
            <td>{into_view_option(workflow_run_task.output)}</td>
            <td>
            {
                match workflow_run_task.rules {
                    Some(rules) => format!("{:?}", rules),
                    None => "-".to_owned()
                }
            }
            </td>
            <td>{into_view_option(workflow_run_task.task_start)}</td>
            <td>{into_view_option(workflow_run_task.task_end)}</td>
            <td>{into_view_option(workflow_run_task.progress)}</td>
        </tr>
    }
}

#[component]
fn WorkflowRun(cx: Scope, workflow_run: WorkflowRun) -> impl IntoView {
    let details_id = format!("tasks{}", workflow_run.workflow_run_id);
    let actions = match workflow_run.status {
        WorkflowRunStatus::Waiting => Some(view! { cx,
            <RowAction
                title="Schedule Workflow Run"
                api_url=format!("/api/workflow-engine/workflow-runs/schedule/{}", workflow_run.workflow_run_id)
                icon="fa-play"/>
        }),
        WorkflowRunStatus::Running => Some(view! { cx,
            <RowAction
                title="Cancel Workflow Run"
                api_url=format!("/api/workflow-engine/workflow-runs/cancel/{}", workflow_run.workflow_run_id)
                icon="fa-stop"/>
        }),
        WorkflowRunStatus::Failed | WorkflowRunStatus::Canceled => Some(view! { cx,
            <RowAction
                title="Restart Workflow Run"
                api_url=format!("/api/workflow-engine/workflow-runs/restart/{}", workflow_run.workflow_run_id)
                icon="fa-rotate-right"/>
        }),
        WorkflowRunStatus::Complete | WorkflowRunStatus::Scheduled | WorkflowRunStatus::Paused => {
            None
        }
    };
    view! { cx,
        <RowWithDetails
            details_id=details_id
            column_count=7
            details_header=view! { cx,
                <tr>
                    <th>"Order"</th>
                    <th>"Task ID"</th>
                    <th>"Name"</th>
                    <th>"Description"</th>
                    <th>"Status"</th>
                    <th>"Parameters"</th>
                    <th>"Output"</th>
                    <th>"Rules"</th>
                    <th>"Start"</th>
                    <th>"End"</th>
                    <th>"Progress"</th>
                </tr>
            }
            details=workflow_run.tasks
            details_row_builder=|cx, task| view! { cx, <WorkflowRunTask workflow_run_task=task/> }
        >
            <td>{into_view(workflow_run.workflow_run_id)}</td>
            <td>{into_view(workflow_run.workflow_id)}</td>
            <td>{into_view(workflow_run.status)}</td>
            <td>{into_view_option(workflow_run.executor_id)}</td>
            <td>{into_view_option(workflow_run.progress)}</td>
            <td>
                {actions}
                <RowAction
                    title="Enter Workflow Run"
                    api_url=format!("/api/workflow-engine/workflow-run/{}", workflow_run.workflow_run_id)
                    icon="fa-right-to-bracket"/>
            </td>
        </RowWithDetails>
    }
}

const WORKFLOW_RUNS_TABLE_ID: &str = "active-workflow-runs-tbl";

#[component]
pub fn ActiveWorkflowRuns(cx: Scope, workflow_runs: Vec<WorkflowRun>) -> impl IntoView {
    view! { cx,
        <DataTableExtras
            id=WORKFLOW_RUNS_TABLE_ID
            caption="Active Workflow Runs"
            header=view! { cx,
                <tr>
                    <th>"Tasks"</th>
                    <th>"ID"</th>
                    <th>"Workflow ID"</th>
                    <th>"Status"</th>
                    <th>"Executor ID"</th>
                    <th>"Progress"</th>
                    <th>"Actions"</th>
                </tr>
            }
            items=workflow_runs
            row_builder=|cx, workflow_run| view! { cx, <WorkflowRun workflow_run=workflow_run/> }
            data_source=WorkflowEngineMainPageTabs::WorkflowRuns.get_url().trim_end_matches("/tab").to_owned()
            refresh=true
            extra_buttons=vec![
                ExtraTableButton::new(
                    "New Workflow Run",
                    "/api/workflow-engine/workflow-runs/init-modal",
                    "fa-plus"
                )
                .add_target(ADD_MODAL_TARGET)
                .add_swap(ADD_MODAL_SWAP)
            ]/>
    }
}

#[component]
pub fn ActiveWorkflowRunsTab(cx: Scope, workflow_runs: Vec<WorkflowRun>) -> impl IntoView {
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::WorkflowRuns/>
        <ActiveWorkflowRuns workflow_runs=workflow_runs/>
    }
}

#[component]
fn Executor(cx: Scope, executor: Executor) -> impl IntoView {
    let actions = if executor.session_active {
        let cancel_post: String = format!(
            "/api/workflow-engine/executors/cancel/{}",
            executor.executor_id
        );
        let shutdown_post: String = format!(
            "/api/workflow-engine/executors/shutdown/{}",
            executor.executor_id
        );
        Some(view! { cx,
            <RowAction title="Cancel Executor" api_url=cancel_post icon="fa-stop"/>
            <RowAction title="Shutdown Executor" api_url=shutdown_post icon="fa-power-off"/>
        })
    } else {
        None
    };
    view! { cx,
        <tr>
            <td>{into_view(executor.executor_id)}</td>
            <td>{into_view(executor.pid)}</td>
            <td>{into_view(executor.username)}</td>
            <td>{into_view(executor.application_name)}</td>
            <td>{into_view(executor.client_addr)}</td>
            <td>{into_view(executor.client_port)}</td>
            <td>{into_view(executor.exec_start)}</td>
            <td>{into_view(executor.session_active)}</td>
            <td>{into_view(executor.workflow_run_count)}</td>
            <td>{actions}</td>
        </tr>
    }
}

#[component]
pub fn ActiveExecutors(cx: Scope, executors: Vec<Executor>) -> impl IntoView {
    view! { cx,
        <DataTableExtras
            id="active-executors-tbl"
            caption="Active Executors"
            header=view! { cx,
                <tr>
                    <th>"ID"</th>
                    <th>"PID"</th>
                    <th>"Username"</th>
                    <th>"Application"</th>
                    <th>"Client Address"</th>
                    <th>"Client Port"</th>
                    <th>"Start"</th>
                    <th>"Active"</th>
                    <th>"Workflow Run Count"</th>
                    <th>"Actions"</th>
                </tr>
            }
            items=executors
            row_builder=|cx, executor| view! { cx, <Executor executor=executor/> }
            data_source=WorkflowEngineMainPageTabs::Executors.get_url().trim_end_matches("/tab").to_owned()
            refresh=true
            extra_buttons=vec![
                ExtraTableButton::new(
                    "Clean Executors",
                    "/api/workflow-engine/executors/clean",
                    "fa-broom"
                ),
            ]/>
    }
}

#[component]
pub fn ActiveExecutorsTab(cx: Scope, executors: Vec<Executor>) -> impl IntoView {
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::Executors/>
        <ActiveExecutors executors=executors/>
    }
}

#[component]
fn ScheduledJob(
    cx: Scope,
    job_id: JobId,
    workflow_id: WorkflowId,
    workflow_name: String,
    maintainer: String,
    is_paused: bool,
    next_run: NaiveDateTime,
    current_workflow_run_id: Option<WorkflowRunId>,
    workflow_run_status: Option<WorkflowRunStatus>,
    executor_id: Option<ExecutorId>,
    progress: Option<i16>,
    entries: Vec<ScheduleEntry>,
) -> impl IntoView {
    view! { cx,
        <RowWithDetails
            details_id=job_id.to_string()
            details_header=view! { cx,
                <tr>
                    <th>"Day of the Week"</th>
                    <th>"Time of the Day"</th>
                </tr>
            }
            details=entries
            details_row_builder=|cx, entry| view! { cx,
                <tr>
                    <td>{entry.day_of_the_week_display()}</td>
                    <td>{into_view(entry.time_of_day)}</td>
                </tr>
            }
            column_count=13
        >
            <td>{into_view(job_id)}</td>
            <td>{into_view(workflow_id)}</td>
            <td>{workflow_name}</td>
            <td>"Scheduled"</td>
            <td>{maintainer}</td>
            <td>{into_view(is_paused)}</td>
            <td>{into_view(next_run)}</td>
            <td>{into_view_option(current_workflow_run_id)}</td>
            <td>{into_view_option(workflow_run_status)}</td>
            <td>{into_view_option(executor_id)}</td>
            <td>{into_view_option(progress)}</td>
        </RowWithDetails>
    }
}

#[component]
fn IntervalJob(
    cx: Scope,
    job_id: JobId,
    workflow_id: WorkflowId,
    workflow_name: String,
    maintainer: String,
    is_paused: bool,
    next_run: NaiveDateTime,
    current_workflow_run_id: Option<WorkflowRunId>,
    workflow_run_status: Option<WorkflowRunStatus>,
    executor_id: Option<ExecutorId>,
    progress: Option<i16>,
    months: i32,
    days: i32,
    minutes: i32,
) -> impl IntoView {
    view! { cx,
        <RowWithDetails
            details_id=job_id.to_string()
            details_header=view! { cx,
                <tr>
                    <th>"Months"</th>
                    <th>"Days"</th>
                    <th>"Minutes"</th>
                </tr>
            }
            details=vec![(months, days, minutes)]
            details_row_builder=|cx, interval| view! { cx,
                <tr>
                    <td>{interval.0}</td>
                    <td>{interval.1}</td>
                    <td>{interval.2}</td>
                </tr>
            }
            column_count=13
        >
            <td>{into_view(job_id)}</td>
            <td>{into_view(workflow_id)}</td>
            <td>{workflow_name}</td>
            <td>"Interval"</td>
            <td>{maintainer}</td>
            <td>{into_view(is_paused)}</td>
            <td>{into_view(next_run)}</td>
            <td>{into_view_option(current_workflow_run_id)}</td>
            <td>{into_view_option(workflow_run_status)}</td>
            <td>{into_view_option(executor_id)}</td>
            <td>{into_view_option(progress)}</td>
        </RowWithDetails>
    }
}

#[component]
fn JobRow(cx: Scope, job: Job) -> impl IntoView {
    match job.job_type {
        JobType::Scheduled { entries } => view! { cx,
            <ScheduledJob
                job_id=job.job_id
                workflow_id=job.workflow_id
                workflow_name=job.workflow_name
                maintainer=job.maintainer
                is_paused=job.is_paused
                next_run=job.next_run
                current_workflow_run_id=job.current_workflow_run_id
                workflow_run_status=job.workflow_run_status
                executor_id=job.executor_id
                progress=job.progress
                entries=entries/>
        },
        JobType::Interval { interval } => {
            let minutes = (interval.microseconds / 60 * 1_000_000) as i32;
            view! { cx,
                <IntervalJob
                    job_id=job.job_id
                    workflow_id=job.workflow_id
                    workflow_name=job.workflow_name
                    maintainer=job.maintainer
                    is_paused=job.is_paused
                    next_run=job.next_run
                    current_workflow_run_id=job.current_workflow_run_id
                    workflow_run_status=job.workflow_run_status
                    executor_id=job.executor_id
                    progress=job.progress
                    months=interval.months
                    days=interval.days
                    minutes=minutes/>
            }
        }
    }
}

const JOBS_TABLE_ID: &str = "jobs-tbl";

#[component]
pub fn Jobs(cx: Scope, jobs: Vec<Job>) -> impl IntoView {
    view! { cx,
        <DataTableExtras
            id=JOBS_TABLE_ID
            caption="Jobs"
            header=view! { cx,
                <tr>
                    <th rowspan=2>"Details"</th>
                    <th rowspan=2>"ID"</th>
                    <th rowspan=2>"Workflow ID"</th>
                    <th rowspan=2>"Workflow Name"</th>
                    <th rowspan=2>"Type"</th>
                    <th rowspan=2>"Maintainer"</th>
                    <th rowspan=2>"Paused?"</th>
                    <th rowspan=2>"Next Run"</th>
                    <th colspan=4>"Current Workflow Run"</th>
                </tr>
                <tr>
                    <th>"ID"</th>
                    <th>"Status"</th>
                    <th>"Executor ID"</th>
                    <th>"Progress"</th>
                </tr>
            }
            items=jobs
            row_builder=|cx, job| view! { cx, <JobRow job=job/> }
            data_source=WorkflowEngineMainPageTabs::Jobs.get_url().trim_end_matches("/tab").to_owned()
            refresh=true
            extra_buttons=vec![
                ExtraTableButton::new(
                    "Create new Job",
                    "/api/workflow-engine/jobs/create-modal",
                    "fa-plus"
                )
                .add_target(ADD_MODAL_TARGET)
                .add_swap(ADD_MODAL_SWAP)
            ]/>
    }
}

#[component]
pub fn JobsTab(cx: Scope, jobs: Vec<Job>) -> impl IntoView {
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::Jobs/>
        <Jobs jobs=jobs/>
    }
}

#[derive(EnumIter, PartialEq)]
pub enum WorkflowEngineMainPageTabs {
    Executors,
    WorkflowRuns,
    Jobs,
}

impl WorkflowEngineMainPageTabs {
    fn id(&self) -> &'static str {
        match self {
            Self::Executors => "executors-tab",
            Self::WorkflowRuns => "workflow-runs-tab",
            Self::Jobs => "jobs-tab",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Executors => "Executors",
            Self::WorkflowRuns => "Workflow Runs",
            Self::Jobs => "Jobs",
        }
    }

    fn get_url(&self) -> &'static str {
        match self {
            Self::Executors => "/api/workflow-engine/executors/tab",
            Self::WorkflowRuns => "/api/workflow-engine/workflow-runs/tab",
            Self::Jobs => "/api/workflow-engine/jobs/tab",
        }
    }

    fn tabs_view(&self, cx: Scope, is_selected: bool) -> impl IntoView {
        let selected = if is_selected { "true" } else { "false" };
        let class = if is_selected {
            "nav-link active"
        } else {
            "nav-link"
        };
        view! { cx,
            <li class="nav-item" role="presentation">
                <button class=class id=self.id() type="button" role="tab" aria-selected=selected hx-get=self.get_url()>
                    {self.label()}
                </button>
            </li>
        }
    }
}

#[component]
fn Tabs(cx: Scope, selected_tab: WorkflowEngineMainPageTabs) -> impl IntoView {
    view! { cx,
        <ul class="nav nav-tabs" id="tabs" role="tablist">
            {WorkflowEngineMainPageTabs::iter()
                .map(|t| {
                    let is_selected = selected_tab == t;
                    t.tabs_view(cx, is_selected)
                })
                .collect::<Vec<_>>()}
        </ul>
    }
}

pub fn default_workflow_engine_tab_url() -> &'static str {
    WorkflowEngineMainPageTabs::Executors.get_url()
}

#[component]
fn WorkflowOptions(cx: Scope, workflows: Vec<Workflow>) -> impl IntoView {
    let options = workflows
        .into_iter()
        .enumerate()
        .map(|(i, w)| {
            view! { cx,
                <option value=w.workflow_id.to_string() selected=i == 0>{w.name}</option>
            }
        })
        .collect_view(cx);
    view! { cx,
        <select class="form-select" id="workflow" name="workflow_id" required>
            {options}
        </select>
    }
}

#[component]
pub fn NewWorkflowRunModal(cx: Scope, workflows: Vec<Workflow>) -> impl IntoView {
    view! { cx,
        <CreateModal
            id="initWorkflowRun"
            title="New Workflow Run"
            target=format!("#{WORKFLOW_RUNS_TABLE_ID}Container")
            form=view! { cx,
                <div class="row mb-3">
                    <label for="workflow" class="col-sm-3 col-form-label">"Workflow"</label>
                    <div class="col-sm-9">
                        <WorkflowOptions workflows=workflows/>
                    </div>
                </div>
            }
            post_url="/api/workflow-engine/workflow-runs/init"/>
    }
}

#[component]
pub fn NewJobNextRun(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="row mb-3">
            <label for="next_run" class="col-sm-3 col-form-label">"Next Run"</label>
            <div class="col-sm-9">
                <input class="form-control" type="datetime-local" name="next_run" id="next_run" required/>
            </div>
        </div>
    }
}

#[component]
pub fn JobScheduleEntry(cx: Scope) -> impl IntoView {
    view! { cx,
        <Row class="schedule-entry mb-1">
            <Col>
                <select name="day_of_the_week">
                    <option value="1" selected>"Monday"</option>
                    <option value="2">"Tuesday"</option>
                    <option value="3">"Wednesday"</option>
                    <option value="4">"Thursday"</option>
                    <option value="5">"Friday"</option>
                    <option value="6">"Saturday"</option>
                    <option value="7">"Sunday"</option>
                </select>
            </Col>
            <Col>
                <input name="time_of_day" type="time"/>
            </Col>
            <Col>
                <button class="btn btn-primary" onclick="removeJobScheduleEntry(this)">
                    <i class="fa-solid fa-minus"></i>
                </button>
            </Col>
        </Row>
    }
}

#[component]
pub fn NewScheduledJob(cx: Scope) -> impl IntoView {
    view! { cx,
        <div id="jobSchedule">
            <Row class="mb-1">
                <Col size=8>
                    <label>"Schedule Entries"</label>
                </Col>
                <Col size=4>
                    <button class="btn btn-primary" hx-get="/api/workflow-engine/jobs/job-schedule-entry"
                        hx-target="#scheduleEntries" hx-swap="beforeend"
                    >
                        <i class="fa-solid fa-plus"></i>
                    </button>
                </Col>
            </Row>
            <div id="scheduleEntries"></div>
        </div>
    }
}

#[component]
pub fn NewIntervalJob(cx: Scope) -> impl IntoView {
    view! { cx,
        <Row>
            <Row>
                <Col>
                    <label class="form-label" for="months">"Months"</label>
                </Col>
                <Col>
                    <input class="form-control" id="months" name="months" type="text"/>
                </Col>
            </Row>
            <Row>
                <Col>
                    <label class="form-label" for="days">"Days"</label>
                </Col>
                <Col>
                    <input class="form-control" id="days" name="days" type="text"/>
                </Col>
            </Row>
            <Row>
                <Col>
                    <label class="form-label" for="minutes">"Minutes"</label>
                </Col>
                <Col>
                    <input class="form-control" id="minutes" name="minutes" type="text"/>
                </Col>
            </Row>
        </Row>
    }
}

#[component]
pub fn NewJobModal(cx: Scope, workflows: Vec<Workflow>) -> impl IntoView {
    view! { cx,
        <CreateModal
            id="createJob"
            title="Create Job"
            target=format!("#{JOBS_TABLE_ID}Container")
            form=view! { cx,
                <div class="row mb-3">
                    <label for="workflow" class="col-sm-3 col-form-label">"Workflow"</label>
                    <div class="col-sm-9">
                        <WorkflowOptions workflows=workflows/>
                    </div>
                </div>
                <div class="row mb-3">
                    <label for="maintainer" class="col-sm-3 col-form-label">"Maintainer"</label>
                    <div class="col-sm-9">
                        <input class="form-control" type="text" name="maintainer" id="maintainer"/>
                    </div>
                </div>
                <div class="row mb-3 ms-2">
                    <div class="form-check col">
                        <input class="form-check-input" type="radio" name="job_type"
                            id="jobTypeScheduled" value="scheduled"
                            hx-get="/api/workflow-engine/jobs/job-type"
                            hx-target="#jobTypeContainer" hx-swap="innerHTML" checked/>
                        <label class="form-check-label" for="jobTypeScheduled">
                            "Scheduled"
                        </label>
                    </div>
                    <div class="form-check col">
                        <input class="form-check-input" type="radio" name="job_type"
                            id="jobTypeInterval" value="interval"
                            hx-get="/api/workflow-engine/jobs/job-type"
                            hx-target="#jobTypeContainer" hx-swap="innerHTML"/>
                        <label class="form-check-label" for="jobTypeInterval">
                            "Interval"
                        </label>
                    </div>
                </div>
                <fieldset id="jobTypeContainer" class="text-center">
                    <NewScheduledJob/>
                </fieldset>
                <div class="row mb-3">
                    <div class="form-check ms-2">
                        <input class="form-check-input" type="checkbox" name="next_run_chk"
                            id="next_run_chk" value="" hx-get="/api/workflow-engine/jobs/next-run"
                            hx-target="#nextRunContainer" hx-swap="innerHTML"/>
                        <label for="next_run_chk" class="form-check-label">"Set Next Run?"</label>
                    </div>
                    <div id="nextRunContainer"></div>
                </div>
            }
            post_url="/api/workflow-engine/jobs"/>
    }
}
