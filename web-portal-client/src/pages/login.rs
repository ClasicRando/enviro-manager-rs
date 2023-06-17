use leptos::*;
use web_portal_common::Credentials;

use crate::{
    api::{self, AuthorizedApi, UnauthorizedApi},
    components::credentials::CredentialsForm,
};

#[component]
pub fn Login<F>(cx: Scope, api: UnauthorizedApi, on_success: F) -> impl IntoView
where
    F: Fn(AuthorizedApi) + 'static + Clone + Copy,
{
    let (login_error, set_login_error) = create_signal(cx, None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(cx, false);
    let login_action = create_action(cx, move |(email, password): &(String, String)| {
        let credentials = Credentials {
            email: email.clone(),
            password: password.clone(),
        };
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = api.login(&credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(res) => {
                    set_login_error.update(|e| *e = None);
                    on_success(res);
                }
                Err(err) => {
                    let msg = match err {
                        api::Error::Fetch(js_err) => {
                            format!("{js_err:?}")
                        }
                        api::Error::ApiError(err) => err,
                        _ => format!("{}", err),
                    };
                    error!("Unable to login with {}: {msg}", credentials.email);
                    set_login_error.update(|e| *e = Some(msg));
                }
            }
        }
    });
    let disabled = Signal::derive(cx, move || wait_for_response.get());
    view! { cx,
        <CredentialsForm
            action=login_action
            error=login_error.into()
            disabled/>
    }
}
