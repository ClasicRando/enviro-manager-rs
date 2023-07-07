use maud::{html, PreEscaped};
use workflow_engine::executor::data::Executor;

pub fn active_executors(executors: Vec<Executor>) -> PreEscaped<String> {
    html! {
        table .table .table-striped .caption-top {
            caption { "Active Executors" }
            thead {
                tr {
                    th { "ID" }
                    th { "PID" }
                    th { "Username" }
                    th { "Application" }
                    th { "Client Address" }
                    th { "Client Port" }
                    th { "Start" }
                    th { "Active?" }
                    th { "Workflow Run Count" }
                }
            }
            tbody {
                @for executor in executors {
                    tr {
                        td { (executor.executor_id) }
                        td { (executor.pid) }
                        td { (executor.username) }
                        td { (executor.application_name) }
                        td { (executor.client_addr) }
                        td { (executor.client_port) }
                        td { (executor.exec_start) }
                        td { (executor.session_active) }
                        td { (executor.workflow_run_count) }
                    }
                }
            }
        }
    }
}
