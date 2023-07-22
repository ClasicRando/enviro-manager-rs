use leptos::*;
use strum::{EnumIter, IntoEnumIterator};
use workflow_engine::{
    executor::data::Executor,
    workflow_run::data::{WorkflowRun, WorkflowRunStatus, WorkflowRunTask},
};

use crate::components::{
    into_view, into_view_option,
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

#[component]
pub fn ActiveWorkflowRuns(cx: Scope, workflow_runs: Vec<WorkflowRun>) -> impl IntoView {
    view! { cx,
        <DataTableExtras
            id="active-workflow-runs-tbl"
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
            data_source=WorkflowEngineMainPageTabs::WorkflowRuns.get_url().trim_end_matches("-tab").to_owned()
            refresh=true
            extra_buttons=vec![
                ExtraTableButton::new(
                    "New Workflow Run",
                    "#",
                    "fa-plus"
                )
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
            }
            items=executors
            row_builder=|cx, executor| view! { cx, <Executor executor=executor/> }
            data_source=WorkflowEngineMainPageTabs::Executors.get_url().trim_end_matches("-tab").to_owned()
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

#[derive(EnumIter, PartialEq)]
pub enum WorkflowEngineMainPageTabs {
    Executors,
    WorkflowRuns,
}

impl WorkflowEngineMainPageTabs {
    fn id(&self) -> &'static str {
        match self {
            Self::Executors => "executors-tab",
            Self::WorkflowRuns => "workflow-runs-tab",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Executors => "Executors",
            Self::WorkflowRuns => "Workflow Runs",
        }
    }

    fn get_url(&self) -> &'static str {
        match self {
            Self::Executors => "/api/workflow-engine/executors-tab",
            Self::WorkflowRuns => "/api/workflow-engine/workflow-runs-tab",
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
