use actix_session::Session;
use common::api::{validate_session, ApiResponse, QueryApiFormat};

use crate::service::roles::{Role, RoleService};

/// API endpoint to fetch all roles
pub async fn roles<R>(
    session: Session,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<Role>>
where
    R: RoleService,
{
    let query = query.into_inner();
    let uuid = match validate_session(&session, query.f) {
        Ok(inner) => inner,
        Err(response) => return response,
    };
    match service.read_all(&uuid).await {
        Ok(roles) => ApiResponse::success(roles, query.f),
        Err(error) => ApiResponse::error(error, query.f),
    }
}
