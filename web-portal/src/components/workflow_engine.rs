use leptos::*;
use strum::{EnumIter, IntoEnumIterator};
use workflow_engine::{
    executor::data::Executor,
    workflow_run::data::{WorkflowRun, WorkflowRunTask},
};

use super::{
    base::BasePage,
    into_view, option_into_view,
    table::{DataTable, DataTableExtras, ExtraTableButton, RowWithDetails},
};

#[component]
fn WorkflowRunTask(cx: Scope, workflow_run_task: WorkflowRunTask) -> impl IntoView {
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

#[component]
fn WorkflowRun(cx: Scope, workflow_run: WorkflowRun) -> impl IntoView {
    let details_id = format!("tasks{}", workflow_run.workflow_run_id);
    view! { cx,
        <RowWithDetails
            details_id=details_id
            column_count=6
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
            <td>{option_into_view(workflow_run.executor_id)}</td>
            <td>{option_into_view(workflow_run.progress)}</td>
        </RowWithDetails>
    }
}

#[component]
pub fn ActiveWorkflowRuns(cx: Scope, workflow_runs: Vec<WorkflowRun>) -> impl IntoView {
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::WorkflowRuns/>
        <DataTable
            id="active-workflow-runs-tbl"
            caption="Active Workflow Runs"
            header=view! { cx,
                <th>"Details"</th>
                <th>"ID"</th>
                <th>"Workflow ID"</th>
                <th>"Status"</th>
                <th>"Executor ID"</th>
                <th>"Progress"</th>
            }
            items=workflow_runs
            row_builder=|cx, workflow_run| view! { cx, <WorkflowRun workflow_run=workflow_run/> }
            data_source=WorkflowEngineMainPageTabs::WorkflowRuns.get_url()
            refresh=true/>
    }
}

#[component]
fn Executor(cx: Scope, executor: Executor) -> impl IntoView {
    let cancel_post: String = format!(
        "/api/workflow-engine/executors/cancel/{}",
        executor.executor_id
    );
    let shutdown_post: String = format!(
        "/api/workflow-engine/executors/shutdown/{}",
        executor.executor_id
    );
    let actions = if executor.session_active {
        Some(view! { cx,
            <i class="table-action fa-solid fa-stop fa-xl me-2 px-1" hx-trigger="click" hx-post=cancel_post></i>
            <i class="table-action fa-solid fa-power-off fa-xl px-1" hx-trigger="click" hx-post=shutdown_post></i>
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
            <td>
                {actions}
            </td>
        </tr>
    }
}

#[component]
pub fn ActiveExecutors(cx: Scope, executors: Vec<Executor>) -> impl IntoView {
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::Executors/>
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
            data_source=WorkflowEngineMainPageTabs::Executors.get_url()
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
pub fn workflow_engine(cx: Scope, user_full_name: String) -> impl IntoView {
    view! { cx,
        <BasePage
            title="Index"
            user_full_name=user_full_name
        >
            <div id="tabs" hx-get={WorkflowEngineMainPageTabs::Executors.get_url()} hx-trigger="load"
                hx-target="#tabs" hx-swap="innerHTML"></div>
        </BasePage>
    }
}
