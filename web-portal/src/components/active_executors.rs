use leptos::*;
use workflow_engine::executor::data::Executor;

use super::into_view;

#[component]
pub fn executor(cx: Scope, executor: Executor) -> impl IntoView {
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
                    .map(|ex| view! { cx, <Executor executor=ex/> })
                    .collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}
