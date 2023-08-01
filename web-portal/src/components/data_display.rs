use std::fmt::Display;

use leptos::*;

#[component]
pub fn DataField<T>(
    cx: Scope,
    id: &'static str,
    label: &'static str,
    column_width: u8,
    data: T,
) -> impl IntoView
where
    T: Display,
{
    view! { cx,
        <label class="col-sm-1 col-form-label" for=id>{label}</label>
        <div class=format!("col-sm-{column_width}")>
            <input id=id class="data-field form-control" value=format!("{data}")/>
        </div>
    }
}

#[component]
pub fn DataDisplay<S, IV, IV2>(
    cx: Scope,
    id: &'static str,
    title: S,
    #[prop(optional)] fields: Option<IV>,
    #[prop(optional)] table: Option<IV2>,
    #[prop(optional)] refresh: Option<String>,
) -> impl IntoView
where
    S: Into<String>,
    IV: IntoView,
    IV2: IntoView,
{
    let refresh_button = refresh.map(|data_source| {
        view! { cx,
            <button type="button" title="Refresh" class="btn btn-secondary"
                hx-get=data_source hx-target=format!("#{id}") hx-swap="outerHTML">
                <i class="fa-solid fa-refresh"></i>
            </button>
        }
    });
    view! { cx,
        <div id=id>
            <div class="btn-toolbar mt-1" role="toolbar">
                <h3>{title.into()}</h3>
                <div class="btn-group ms-auto">
                {refresh_button}
                </div>
            </div>
            <hr class="border border-primary border-3 opacity-75 mt-1" />
            <div class="my-1">
                <fieldset disabled>
                    {fields}
                </fieldset>
            </div>
            {table}
        </div>
    }
}
