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
            <input id=id class="form-control" value=format!("{data}")/>
        </div>
    }
}

#[component]
pub fn DataDisplay(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <fieldset disabled>
            {children(cx)}
        </fieldset>
    }
}
