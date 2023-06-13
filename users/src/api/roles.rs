use common::api::{request::ApiRequest, ApiResponse, QueryApiFormat};

use super::UidRequest;
use crate::service::roles::{Role, RoleService};

/// API endpoint to fetch all roles
pub async fn roles<R>(
    api_request: ApiRequest<UidRequest>,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<Role>>
where
    R: RoleService,
{
    let query = query.into_inner();
    let UidRequest { uid } = api_request.into_inner();
    match service.read_all(&uid).await {
        Ok(roles) => ApiResponse::success(roles, query.f),
        Err(error) => ApiResponse::error(error, query.f),
    }
}
