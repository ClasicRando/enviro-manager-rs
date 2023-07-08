use leptos::*;
use strum::{EnumIter, IntoEnumIterator};
use workflow_engine::{
    executor::data::Executor,
    workflow_run::data::{WorkflowRun, WorkflowRunTask},
};

use super::{into_view, option_into_view, BasePage};

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

#[component]
fn workflow_run(cx: Scope, workflow_run: WorkflowRun) -> impl IntoView {
    let details_id = format!("tasks{}", workflow_run.workflow_run_id);
    let hm_on = format!(
        "click: toggleDisplay(document.getElementById('{}'))",
        details_id
    );
    view! { cx,
        <tr>
            <td>
                <button class="btn btn-primary" hx-on=hm_on>
                    <i class="fa-solid fa-plus"></i>
                </button>
            </td>
            <td>{into_view(workflow_run.workflow_run_id)}</td>
            <td>{into_view(workflow_run.workflow_id)}</td>
            <td>{into_view(workflow_run.status)}</td>
            <td>{option_into_view(workflow_run.executor_id)}</td>
            <td>{option_into_view(workflow_run.progress)}</td>
        </tr>
        <tr id=details_id class="d-none">
            <td colspan="6">
                <table class="table table-stripped">
                    <thead>
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
                    </thead>
                    <tbody>
                        {workflow_run.tasks.into_iter()
                            .map(|wrt| view! { cx, <WorkflowRunTask workflow_run_task=wrt/> })
                            .collect::<Vec<_>>()}
                    </tbody>
                </table>
            </td>
        </tr>
    }
}

#[component]
pub fn active_workflow_runs(cx: Scope, workflow_runs: Vec<WorkflowRun>) -> impl IntoView {
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::WorkflowRuns/>
        <div class="table-responsive-sm">
            <table class="table table-stripped caption-top">
                <caption>"Active Workflow Runs"</caption>
                <thead>
                    <tr>
                        <th>"Details"</th>
                        <th>"ID"</th>
                        <th>"Workflow ID"</th>
                        <th>"Status"</th>
                        <th>"Executor ID"</th>
                        <th>"Progress"</th>
                    </tr>
                </thead>
                <tbody>
                    {workflow_runs.into_iter()
                        .map(|wr| view! { cx, <WorkflowRun workflow_run=wr/> })
                        .collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
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

#[component]
pub fn active_executors(cx: Scope, executors: Vec<Executor>) -> impl IntoView {
    view! { cx,
        <Tabs selected_tab=WorkflowEngineMainPageTabs::Executors/>
        <div class="table-responsive-sm">
            <table class="table table-striped caption-top">
                <caption>"Active Executors"</caption>
                <thead>
                    <tr>
                        <th>"ID"</th>
                        <th>"PID"</th>
                        <th>"Username"</th>
                        <th>"Application"</th>
                        <th>"Client Address"</th>
                        <th>"Client Port"</th>
                        <th>"Start"</th>
                        <th>"Active?"</th>
                        <th>"Workflow Run Count"</th>
                    </tr>
                </thead>
                <tbody>
                    {executors.into_iter()
                        .map(|ex| view! { cx, <tr><Executor executor=ex/></tr> })
                        .collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
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
            Self::Executors => "/api/htmx/workflow-engine/executors",
            Self::WorkflowRuns => "/api/htmx/workflow-engine/workflow-runs",
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
                <button class=class id=self.id() type="button" role="tab" aria-selected=selected
                    hx-get=self.get_url() hx-swap="innerHTML">{self.label()}</button>
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
