use leptos::*;

#[component]
pub fn row_with_details(
    cx: Scope,
    children: Children,
    details_id: String,
    detail_columns: &'static [&'static str],
    detail_rows: View,
) -> impl IntoView {
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
            {children(cx)}
        </tr>
        <DetailsTable details_id=details_id columns=detail_columns rows=detail_rows/>
    }
}

#[component]
pub fn details_table(
    cx: Scope,
    details_id: String,
    columns: &'static [&'static str],
    rows: View,
) -> impl IntoView {
    view! { cx,
        <tr id=details_id class="d-none">
            <td colspan=columns.len()>
                <table class="table table-stripped">
                    <thead>
                        <tr>
                        {columns.iter()
                            .map(|c| view! { cx, <th>{c}</th> })
                            .collect::<Vec<_>>()}
                        </tr>
                    </thead>
                    <tbody>
                    {rows}
                    </tbody>
                </table>
            </td>
        </tr>
    }
}

#[component]
pub fn data_table(
    cx: Scope,
    caption: &'static str,
    columns: &'static [&'static str],
    rows: View,
) -> impl IntoView {
    view! { cx,
        <div class="table-responsive-sm">
            <div>

            </div>
            <table class="table table-striped caption-top">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                    {columns.iter()
                        .map(|c| view! { cx, <th>{c}</th> })
                        .collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody>
                {rows}
                </tbody>
            </table>
        </div>
    }
}
