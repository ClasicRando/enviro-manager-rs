use actix_session::Session;
use common::api::{validate_session, ApiResponse};

use crate::service::roles::{Role, RoleService};

/// API endpoint to fetch all roles
pub async fn roles<R>(session: Session, service: actix_web::web::Data<R>) -> ApiResponse<Vec<Role>>
where
    R: RoleService,
{
    let uuid = match validate_session(&session) {
        Ok(inner) => inner,
        Err(response) => return response,
    };
    match service.read_all(&uuid).await {
        Ok(roles) => ApiResponse::success(roles),
        Err(error) => ApiResponse::error(error),
    }
}
