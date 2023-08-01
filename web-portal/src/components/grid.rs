use leptos::*;

#[component]
pub fn Row(cx: Scope, children: Children, #[prop(optional)] class: &'static str) -> impl IntoView {
    let class = if class.is_empty() {
        "row".to_owned()
    } else {
        format!("row {class}")
    };
    view! { cx,
        <div class=class>
        {children(cx)}
        </div>
    }
}

#[component]
pub fn Col(
    cx: Scope,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional)] size: u8,
) -> impl IntoView {
    let class = if size > 0 {
        format!("col-{size}")
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
