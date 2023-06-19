pub mod home;
#[cfg(feature = "ssr")]
pub mod login;
#[cfg(not(feature = "ssr"))]
pub mod login {
    use leptos::*;

    #[component]
    pub fn Login(cx: Scope) -> impl IntoView {
        view! { cx, <p>"SSR must be enabled"</p> }
    }
}
