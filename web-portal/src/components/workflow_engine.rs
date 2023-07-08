use leptos::*;

use super::BasePage;

#[component]
pub fn workflow_engine(cx: Scope, username: String) -> impl IntoView {
    view! { cx,
        <BasePage
            title="Index"
            username=username
        >
            <ul class="nav nav-tabs" id="tabs" role="tablist">
                <li class="nav-item" role="presentation">
                    <button class="nav-link active" id="executors-tab" hx-on="click: selectTab(this)" type="button" role="tab"
                        aria-controls="executors-tab-pane" aria-selected="true" hx-get="/api/htmx/workflow-engine/executors"
                        hx-trigger="load,click" hx-target="#tableContent" hx-swap="innerHTML">"Executors"</button>
                </li>
                <li class="nav-item" role="presentation">
                    <button class="nav-link" id="workflow-runs-tab" hx-on="click: selectTab(this)" type="button" role="tab"
                        aria-controls="workflow-runs-tab-pane" aria-selected="false"
                        hx-get="/api/htmx/workflow-engine/workflow-runs" hx-trigger="click" hx-target="#tableContent"
                        hx-swap="innerHTML">"Workflow Runs"</button>
                </li>
            </ul>
            <div id="tableContent" class="table-responsive-sm"></div>
        </BasePage>
    }
}
