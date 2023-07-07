use maud::{html, PreEscaped};
use workflow_engine::workflow_run::data::WorkflowRun;

pub fn active_workflow_runs(workflow_runs: Vec<WorkflowRun>) -> PreEscaped<String> {
    html! {
        table .table .table-striped .caption-top {
            caption { "Active Workflow Runs" }
            thead {
                tr {
                    th { "Details" }
                    th { "ID" }
                    th { "Workflow ID" }
                    th { "Status" }
                    th { "Executor ID" }
                    th { "Progress" }
                }
            }
            tbody {
                @for workflow_run in workflow_runs {
                    @let details_id = format!("tasks{}", workflow_run.workflow_run_id);
                    @let hm_on = format!("click: toggleDisplay(document.getElementById('{}'))", details_id);
                    tr {
                        td {
                            button .btn .btn-primary hx-on=(hm_on) {
                                i .fa-solid .fa-plus {}
                            }
                        }
                        td { (workflow_run.workflow_run_id) }
                        td { (workflow_run.workflow_id) }
                        td { (workflow_run.status) }
                        td {
                            @match workflow_run.executor_id {
                                Some(id) => (id),
                                None => "-"
                            }
                        }
                        td {
                            @match workflow_run.progress {
                                Some(val) => (val),
                                None => "-"
                            }
                        }
                    }
                    tr id=(details_id) .d-none {
                        td colspan="6" {
                            table .table .table-striped {
                                thead {
                                    tr {
                                        th { "Order" }
                                        th { "Task ID" }
                                        th { "Name" }
                                        th { "Description" }
                                        th { "Status" }
                                        th { "Parameters" }
                                        th { "Output" }
                                        th { "Rules" }
                                        th { "Start" }
                                        th { "End" }
                                        th { "Progress" }
                                    }
                                }
                                tbody {
                                    @for task in workflow_run.tasks {
                                        tr {
                                            td { (task.task_order) }
                                            td { (task.task_id) }
                                            td { (task.name) }
                                            td { (task.description) }
                                            td { (task.task_status) }
                                             td {
                                                @match task.parameters {
                                                    Some(parameters) => (parameters),
                                                    None => "-"
                                                }
                                            }
                                             td {
                                                @match task.output {
                                                    Some(output) => (output),
                                                    None => "-"
                                                }
                                            }
                                             td {
                                                @match task.rules {
                                                    Some(rules) => ({format!("{:?}", rules)}),
                                                    None => "-"
                                                }
                                            }
                                             td {
                                                @match task.task_start {
                                                    Some(task_start) => (task_start),
                                                    None => "-"
                                                }
                                            }
                                             td {
                                                @match task.task_end {
                                                    Some(task_end) => (task_end),
                                                    None => "-"
                                                }
                                            }
                                             td {
                                                @match task.progress {
                                                    Some(progress) => (progress),
                                                    None => "-"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
