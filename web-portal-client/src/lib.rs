mod api;
mod components;
mod pages;

use leptos::*;
use leptos_router::*;
use web_portal_common::User;

use crate::{
    api::AuthorizedApi,
    components::nav::NavBar,
    pages::{home::Home, login::Login, Page},
};

const WEB_PORTAL_API: &str = "http://127.0.0.1:3000";

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let authorized_api = create_rw_signal(cx, None::<AuthorizedApi>);
    let user_info = create_rw_signal(cx, None::<User>);
    let logged_in = Signal::derive(cx, move || authorized_api.get().is_some());

    let fetch_user_info = create_action(cx, move |_| async move {
        match authorized_api.get() {
            Some(api) => match api.user_info().await {
                Ok(info) => {
                    user_info.update(|i| *i = Some(info));
                }
                Err(err) => {
                    log::error!("Unable to fetch user info: {err}")
                }
            },
            None => {
                log::error!("Unable to fetch user info: not logged in")
            }
        }
    });

    let logout = create_action(cx, move |_| async move {
        match authorized_api.get() {
            Some(api) => match api.logout().await {
                Ok(_) => {
                    authorized_api.update(|a| *a = None);
                    user_info.update(|i| *i = None);
                }
                Err(err) => {
                    log::error!("Unable to logout: {err}")
                }
            },
            None => {
                log::error!("Unable to logout user: not logged in")
            }
        }
    });

    let on_logout = move || {
        logout.dispatch(());
    };

    let unauthorized_api = api::UnauthorizedApi::new(WEB_PORTAL_API);
    // if let Ok(token) = LocalStorage::get(API_TOKEN_STORAGE_KEY) {
    //     let api = api::AuthorizedApi::new(WEB_PORTAL_API, token);
    //     authorized_api.update(|a| *a = Some(api));
    //     fetch_user_info.dispatch(());
    // }

    log::debug!("User is logged in: {}", logged_in.get());

    // create_effect(cx, move |_| {
    //     log::debug!("API authorization state changed");
    //     match authorized_api.get() {
    //         Some(api) => {
    //             log::debug!("API is now authorized: save token in LocalStorage");
    //             LocalStorage::set(API_TOKEN_STORAGE_KEY,
    // api.token()).expect("LocalStorage::set");         }
    //         None => {
    //             log::debug!("API is no longer authorized: delete token from LocalStorage");
    //             LocalStorage::delete(API_TOKEN_STORAGE_KEY);
    //         }
    //     }
    // });

    view! {
        cx,
        <Router>
            <NavBar logged_in on_logout/>
            <Routes>
                <Route
                    path=Page::Home.path()
                    view=move |cx| {
                        view! { cx, <Home user_info=user_info.into()/> }
                    }
                />
                <Route
                    path=Page::Login.path()
                    view=move |cx| {
                        view! { cx,
                            <Login
                                api=unauthorized_api
                                on_success=move |api| {
                                    log::info!("Successfully logged in");
                                    authorized_api.update(|v| *v = Some(api));
                                    let navigate = use_navigate(cx);
                                    navigate(Page::Home.path(), Default::default()).expect("Home route");
                                    fetch_user_info.dispatch(());
                                }/>
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
