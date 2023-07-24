use leptos::*;

use crate::take_if;

pub struct ExtraTableButton {
    title: &'static str,
    api_url: &'static str,
    icon: &'static str,
    target: Option<String>,
    swap: Option<String>,
}

impl ExtraTableButton {
    pub fn new(title: &'static str, api_url: &'static str, icon: &'static str) -> Self {
        Self {
            title,
            api_url,
            icon,
            target: None,
            swap: None,
        }
    }

    pub fn add_target<S>(mut self, target: S) -> Self
    where
        S: Into<String>,
    {
        self.target = Some(target.into());
        self
    }

    pub fn add_swap<S>(mut self, swap: S) -> Self
    where
        S: Into<String>,
    {
        self.swap = Some(swap.into());
        self
    }
}

impl IntoView for ExtraTableButton {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <button title=self.title type="button" class="btn btn-secondary"
                hx-post=self.api_url hx-trigger="click" hx-target=self.target
                hx-swap=self.swap
            >
                <i class=format!("fa-solid {}", self.icon)></i>
            </button>
        }
        .into_view(cx)
    }
}

#[component]
pub fn RowAction<S>(
    cx: Scope,
    title: &'static str,
    api_url: S,
    icon: &'static str,
    #[prop(optional)] target: &'static str,
    #[prop(optional)] swap: &'static str,
) -> impl IntoView
where
    S: Into<String>,
{
    let target = take_if(target, |t| !t.is_empty());
    let swap = take_if(swap, |s| !s.is_empty());
    view! { cx,
        <button class="btn btn-primary me-1" hx-post=api_url.into() title=title hx-target=target hx-swap=swap>
            <i class=format!("fa-solid {icon}")></i>
        </button>
    }
}

#[component]
pub fn RowWithDetails<IV, R, F, IV2>(
    cx: Scope,
    children: Children,
    details_id: String,
    details_header: IV,
    details: Vec<R>,
    details_row_builder: F,
    column_count: u8,
) -> impl IntoView
where
    IV: IntoView,
    F: Fn(Scope, R) -> IV2,
    IV2: IntoView,
{
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
        <DetailsTable
            details_id=details_id
            column_count=column_count
            header=details_header
            items=details
            row_builder=details_row_builder/>
    }
}

#[component]
pub fn DetailsTable<IV, R, F, IV2>(
    cx: Scope,
    details_id: String,
    column_count: u8,
    header: IV,
    items: Vec<R>,
    row_builder: F,
) -> impl IntoView
where
    IV: IntoView,
    F: Fn(Scope, R) -> IV2,
    IV2: IntoView,
{
    let row_elements = items
        .into_iter()
        .map(|row| row_builder(cx, row))
        .collect_view(cx);
    view! { cx,
        <tr id=details_id class="d-none">
            <td colspan=column_count>
                <table class="table table-stripped">
                    <thead>
                        {header}
                    </thead>
                    <tbody>
                    {row_elements}
                    </tbody>
                </table>
            </td>
        </tr>
    }
}

#[component]
pub fn DataTableExtras<IV, R, F, IV2, E>(
    cx: Scope,
    id: &'static str,
    caption: &'static str,
    header: IV,
    items: Vec<R>,
    row_builder: F,
    #[prop(optional)] data_source: String,
    #[prop(optional)] refresh: bool,
    #[prop(optional)] search: bool,
    #[prop(optional)] extra_buttons: E,
) -> impl IntoView
where
    IV: IntoView,
    F: Fn(Scope, R) -> IV2,
    IV2: IntoView,
    E: IntoIterator<Item = ExtraTableButton> + Default,
{
    let container_id = format!("{id}Container");
    let body_id = format!("{id}Body");
    let search_form = if search {
        let search_source = format!("{data_source}/search");
        Some(view! { cx,
            <form role="search" class="d-flex ms-auto">
                <input class="form-control me-2" type="search" placeholder="Search" name="search"
                    aria-label="Search" hx-trigger="keyup changed delay:500ms, search"
                    hx-post=search_source hx-indicator=".htmx-indicator"
                    hx-target=format!("#{body_id}")/>
            </form>
        })
    } else {
        None
    };
    let refresh_button = if refresh {
        Some(view! { cx,
            <button type="button" title="Refresh" class="btn btn-secondary" hx-get=data_source>
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
    let row_elements = items
        .into_iter()
        .map(|row| row_builder(cx, row))
        .collect_view(cx);
    view! { cx,
        <div class="table-responsive-sm" hx-target=format!("#{container_id}") hx-swap="outerHTML"
            id=container_id
        >
            <div class="btn-toolbar mt-1" role="toolbar">
                {search_form}
                <div class=button_group_class>
                {refresh_button}
                {extra_buttons.collect_view(cx)}
                </div>
            </div>
            <table class="table table-striped caption-top" id=id>
                <caption>
                    {caption}
                    <div class="spinner-border htmx-indicator" role="status"></div>
                </caption>
                <thead>{header}</thead>
                <tbody id=body_id>
                    {row_elements}
                </tbody>
            </table>
        </div>
    }
}

#[component]
pub fn DataTable<IV, R, F, IV2>(
    cx: Scope,
    id: &'static str,
    caption: &'static str,
    header: IV,
    items: Vec<R>,
    row_builder: F,
    #[prop(optional)] data_source: String,
    #[prop(optional)] refresh: bool,
    #[prop(optional)] search: bool,
) -> impl IntoView
where
    IV: IntoView,
    F: Fn(Scope, R) -> IV2,
    IV2: IntoView,
{
    view! { cx,
        <DataTableExtras
            id=id
            caption=caption
            header=header
            items=items
            row_builder=row_builder
            extra_buttons=vec![]
            data_source=data_source
            refresh=refresh
            search=search/>
    }
}
