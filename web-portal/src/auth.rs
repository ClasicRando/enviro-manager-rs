use actix_session::Session;
use leptos::*;
use leptos_actix::extract;
use uuid::Uuid;

const SESSION_KEY: &str = "em_uid";

async fn get_session(cx: Scope) -> Result<Session, ServerFnError> {
    extract(cx, |session: Session| async move {
        log::info!("Session {:?}", session.entries());
        session
    })
    .await
}

pub async fn get_uid(cx: Scope) -> Result<Uuid, ServerFnError> {
    let session = get_session(cx).await?;
    match session.get(SESSION_KEY) {
        Ok(Some(uid)) => Ok(uid),
        Ok(None) => Err(ServerFnError::Request(
            "User is not authenticated".to_owned(),
        )),
        Err(error) => Err(ServerFnError::ServerError(format!(
            "Error getting session: {error}"
        ))),
    }
}

pub async fn set_session(cx: Scope, uid: Uuid) -> Result<(), ServerFnError> {
    let session = get_session(cx).await?;
    session
        .insert(SESSION_KEY, uid)
        .map_err(|e| ServerFnError::ServerError(format!("Could not create a new session. {e}")))
}
