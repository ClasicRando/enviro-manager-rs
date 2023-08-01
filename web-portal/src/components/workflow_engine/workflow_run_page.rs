use leptos::*;
use workflow_engine::workflow_run::data::{WorkflowRun, WorkflowRunTask};

use crate::components::{
    data_display::{DataDisplay, DataField},
    grid::Row,
    into_view_option,
    table::DataTable,
    workflow_engine::main_page::WorkflowRunTask,
};

#[component]
fn WorkflowRunTaskTable(cx: Scope, tasks: Vec<WorkflowRunTask>) -> impl IntoView {
    view! { cx,
        <DataTable
            id="workflow_run_tasks"
            caption="Tasks"
            header=view! { cx,
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
            items=tasks
            row_builder=|cx, task| view! { cx,
                <WorkflowRunTask workflow_run_task=task/>
            }/>
    }
}

#[component]
pub fn WorkflowRunDisplay(cx: Scope, workflow_run: WorkflowRun) -> impl IntoView {
    view! { cx,
        <DataDisplay
            id="workflowRunDisplay"
            title="Workflow Run"
            fields=view! { cx,
                <Row class="mb-3">
                    <DataField
                        id="workflow_run_id"
                        label="Workflow Run ID"
                        column_width=2
                        data=workflow_run.workflow_run_id/>
                    <DataField
                        id="workflow_id"
                        label="Workflow ID"
                        column_width=2
                        data=workflow_run.workflow_id/>
                    <DataField
                        id="status"
                        label="Status"
                        column_width=2
                        data=workflow_run.status/>
                </Row>
                <Row class="mb-3">
                    <DataField
                        id="executor_id"
                        label="Executor ID"
                        column_width=2
                        data=into_view_option(workflow_run.executor_id)/>
                    <DataField
                        id="progress"
                        label="Progress"
                        column_width=2
                        data=into_view_option(workflow_run.progress)/>
                </Row>
            }
            table=view! { cx,
                <WorkflowRunTaskTable tasks=workflow_run.tasks/>
            }
            refresh=format!("/api/workflow-engine/workflow-run/{}", workflow_run.workflow_run_id)/>
    }
}
