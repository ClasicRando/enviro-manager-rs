mod components;
mod pages;

use leptos::*;
use leptos_router::*;

use crate::{
    components::nav::NavBar,
    pages::{login::Login, Page},
};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <Router>
            <NavBar/>
                <Routes>
                    // <Route
                    //     path=Page::Home.path()
                    //     view=move |cx| {
                    //         view! { cx, <Home user_info=user_info.into()/> }
                    //     }
                    // />
                    <Route
                        path=Page::Login.path()
                        view=move |cx| {
                            view! { cx,
                                <Login />
                            }
                        }
                    />
                    // <Route
                    //     path=Page::Register.path()
                    //     view=move |cx| {
                    //         view! { cx, <Register api=unauthorized_api/> }
                    //     }
                    // />
                </Routes>
        </Router>
    }
}
