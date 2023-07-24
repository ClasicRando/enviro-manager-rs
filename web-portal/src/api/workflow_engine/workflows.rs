use common::api::ApiResponseBody;
use reqwest::Method;
use workflow_engine::workflow::data::Workflow;

use crate::{utils, ServerFnError};

pub async fn get_workflows() -> Result<Vec<Workflow>, ServerFnError> {
    let workflows_response = utils::api_request(
        format!("http://127.0.0.1:8000/api/v1/workflows?f=msgpack"),
        Method::GET,
        None::<String>,
        None::<()>,
    )
    .await?;
    match workflows_response {
        ApiResponseBody::Success(workflows) => Ok(workflows),
        ApiResponseBody::Message(message) => {
            utils::server_fn_error!("Expected data, got message. {}", message)
        }
        ApiResponseBody::Error(message) | ApiResponseBody::Failure(message) => {
            utils::server_fn_error!(message)
        }
    }
}
