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
