use leptos::*;
use workflow_engine::workflow_run::data::{WorkflowRun, WorkflowRunTask};

use super::{into_view, option_into_view};

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
    }
}
