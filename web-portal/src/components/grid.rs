use leptos::*;

#[component]
pub fn row(cx: Scope, children: Children, #[prop(optional)] size: u8) -> impl IntoView {
    let class = if size > 0 {
        format!("row{size}")
    } else {
        "row".to_owned()
    };
    view! { cx,
        <div class=class>
        {children(cx)}
        </div>
    }
}

#[component]
pub fn col(
    cx: Scope,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional)] size: u8,
) -> impl IntoView {
    let class = if size > 0 {
        format!("col{size}")
    } else {
        "col".to_owned()
    };
    let children = children.map(|c| c(cx).into_view(cx)).unwrap_or_default();
    view! { cx,
        <div class=class>
        {children}
        </div>
    }
}
