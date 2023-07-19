use leptos::*;
use users::data::user::User;
use workflow_engine::workflow_run::data::{WorkflowRun, WorkflowRunId, WorkflowRunTask};

use crate::components::{
    base::BasePage,
    data_display::{DataDisplay, DataField},
    display_option,
    grid::Row,
    table::DataTable,
    workflow_engine::main_page::WorkflowRunTask,
};

#[component]
pub fn WorkflowRunTaskTable(
    cx: Scope,
    workflow_run_id: WorkflowRunId,
    tasks: Vec<WorkflowRunTask>,
) -> impl IntoView {
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
            }
            data_source=format!("/api/workflow-engine/workflow-run/tasks/{workflow_run_id}")
            refresh=true/>
    }
}

#[component]
pub fn WorkflowRunPage(cx: Scope, user: User, workflow_run: WorkflowRun) -> impl IntoView {
    view! { cx,
        <BasePage title="Workflow Run" user=user>
            <div class="my-1">
                <DataDisplay>
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
                            data=display_option(workflow_run.executor_id)/>
                        <DataField
                            id="progress"
                            label="Progress"
                            column_width=2
                            data=display_option(workflow_run.progress)/>
                    </Row>
                </DataDisplay>
            </div>
            <div id="taskTable" hx-swap="innerHTML" hx-target="#taskTable">
                <WorkflowRunTaskTable
                    tasks=workflow_run.tasks
                    workflow_run_id=workflow_run.workflow_run_id/>
            </div>
        </BasePage>
    }
}
