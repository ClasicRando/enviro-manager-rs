use std::sync::LazyLock;

use leptos::*;
use strum::{EnumIter, IntoEnumIterator};
use workflow_engine::{
    executor::data::Executor,
    workflow_run::data::{WorkflowRun, WorkflowRunTask},
};

use super::{
    into_view, option_into_view,
    table::{DataTable, RowWithDetails},
    BasePage,
};

#[component]
fn workflow_run_task(cx: Scope, workflow_run_task: WorkflowRunTask) -> impl IntoView {
    view! { cx,
        <tr>
            <td>{into_view(workflow_run_task.task_order)}</td>
            <td>{into_view(workflow_run_task.task_id)}</td>
            <td>{into_view(workflow_run_task.name)}</td>
            <td>{into_view(workflow_run_task.description)}</td>
            <td>{into_view(workflow_run_task.task_status)}</td>
            <td>{option_into_view(workflow_run_task.parameters)}</td>
            <td>{option_into_view(workflow_run_task.output)}</td>
            <td>
            {
                match workflow_run_task.rules {
                    Some(rules) => format!("{:?}", rules),
                    None => "-".to_owned()
                }
            }
            </td>
            <td>{option_into_view(workflow_run_task.task_start)}</td>
            <td>{option_into_view(workflow_run_task.task_end)}</td>
            <td>{option_into_view(workflow_run_task.progress)}</td>
        </tr>
    }
}

static ACTIVE_WORKFLOW_RUN_TASK_COLUMNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Order",
        "Task ID",
        "Name",
        "Description",
        "Status",
        "Parameters",
        "Output",
        "Rules",
        "Start",
        "End",
        "Progress",
    ]
});

#[component]
fn workflow_run(cx: Scope, workflow_run: WorkflowRun) -> impl IntoView {
    let details_id = format!("tasks{}", workflow_run.workflow_run_id);
    let rows = workflow_run
        .tasks
        .into_iter()
        .map(|wrt| view! { cx, <WorkflowRunTask workflow_run_task=wrt/> })
        .collect_view(cx);
    view! { cx,
        <RowWithDetails
            details_id=details_id
            detail_columns=&ACTIVE_WORKFLOW_RUN_TASK_COLUMNS
            detail_rows=rows
        >
            <td>{into_view(workflow_run.workflow_run_id)}</td>
            <td>{into_view(workflow_run.workflow_id)}</td>
            <td>{into_view(workflow_run.status)}</td>
            <td>{option_into_view(workflow_run.executor_id)}</td>
            <td>{option_into_view(workflow_run.progress)}</td>
        </RowWithDetails>
    }
}

static ACTIVE_WORKFLOW_RUN_COLUMNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Details",
        "ID",
        "Workflow ID",
        "Status",
        "Executor ID",
        "Progress",
    ]
});

#[component]
pub fn active_workflow_runs(cx: Scope, workflow_runs: Vec<WorkflowRun>) -> impl IntoView {
    let rows = workflow_runs
        .into_iter()
        .map(|wr| view! { cx, <WorkflowRun workflow_run=wr/> })
        .collect_view(cx);
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::WorkflowRuns/>
        <DataTable caption="Active Workflow Runs" columns=&ACTIVE_WORKFLOW_RUN_COLUMNS rows=rows/>
    }
}

#[component]
fn executor(cx: Scope, executor: Executor) -> impl IntoView {
    view! { cx,
        <td>{into_view(executor.executor_id)}</td>
        <td>{into_view(executor.pid)}</td>
        <td>{into_view(executor.username)}</td>
        <td>{into_view(executor.application_name)}</td>
        <td>{into_view(executor.client_addr)}</td>
        <td>{into_view(executor.client_port)}</td>
        <td>{into_view(executor.exec_start)}</td>
        <td>{into_view(executor.session_active)}</td>
        <td>{into_view(executor.workflow_run_count)}</td>
    }
}

static ACTIVE_EXECUTOR_COLUMNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "ID",
        "PID",
        "Username",
        "Application",
        "Client Address",
        "Start",
        "Active",
        "Workflow Run Count",
    ]
});

#[component]
pub fn active_executors(cx: Scope, executors: Vec<Executor>) -> impl IntoView {
    let rows = executors
        .into_iter()
        .map(|ex| view! { cx, <tr><Executor executor=ex/></tr> })
        .collect_view(cx);
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::Executors/>
        <DataTable caption="Active Executors" columns=&ACTIVE_EXECUTOR_COLUMNS rows=rows/>
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
            Self::Executors => "/api/workflow-engine/executors",
            Self::WorkflowRuns => "/api/workflow-engine/workflow-runs",
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
fn tabs(cx: Scope, selected_tab: WorkflowEngineMainPageTabs) -> impl IntoView {
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

#[component]
pub fn workflow_engine(cx: Scope, username: String) -> impl IntoView {
    view! { cx,
        <BasePage
            title="Index"
            username=username
        >
            <div id="tabs" hx-get={WorkflowEngineMainPageTabs::Executors.get_url()} hx-trigger="load"
                hx-target="#tabs" hx-swap="innerHTML"></div>
        </BasePage>
    }
}
