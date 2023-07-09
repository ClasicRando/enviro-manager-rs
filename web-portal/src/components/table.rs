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
    id: &'static str,
    caption: &'static str,
    columns: &'static [&'static str],
    rows: View,
    data_source: &'static str,
    #[prop(optional)] refresh: bool,
    #[prop(optional)] search: bool,
) -> impl IntoView {
    let body_id = format!("{id}Body");
    let search_form = if search {
        let search_source = format!("{data_source}/search");
        Some(view! { cx,
            <form role="search" class="d-flex ms-auto">
                <input class="form-control me-2" type="search" placeholder="Search" name="search"
                    aria-label="Search" hx-trigger="keyup changed delay:500ms, search"
                    hx-post=search_source hx-indicator=".htmx-indicator" hx-target=format!("#{body_id}")/>
            </form>
        })
    } else {
        None
    };
    let refresh_button = if refresh {
        Some(view! { cx,
            <button type="button" class="btn btn-secondary" hx-get=data_source>
                <i class="fa-solid fa-refresh"></i>
            </button>
        })
    } else {
        None
    };
    let button_group_class = if search {
        "btn-group"
    } else {
        "btn-group ms-auto"
    };
    view! { cx,
        <div class="table-responsive-sm">
            <div class="btn-toolbar mt-1" role="toolbar">
                {search_form}
                <div class=button_group_class>
                {refresh_button}
                </div>
            </div>
            <table class="table table-striped caption-top" id=id>
                <caption>
                    {caption}
                    <div class="spinner-border htmx-indicator" role="status"></div>
                </caption>
                <thead>
                    <tr>
                    {columns.iter()
                        .map(|c| view! { cx, <th>{c}</th> })
                        .collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody id=body_id>
                {rows}
                </tbody>
            </table>
        </div>
    }
}
