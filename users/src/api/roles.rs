use actix_web_httpauth::extractors::bearer::BearerAuth;
use common::api::{ApiResponse, QueryApiFormat};

use super::{validate_bearer, BearerValidation};
use crate::{data::role::Role, service::roles::RoleService};

/// API endpoint to fetch all roles
pub async fn roles<R>(
    bearer: BearerAuth,
    service: actix_web::web::Data<R>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<Role>>
where
    R: RoleService,
{
    let format = query.into_inner();
    let uid = match validate_bearer(&bearer, format.f) {
        BearerValidation::Valid(uid) => uid,
        BearerValidation::InValid(response) => return response,
    };
    match service.read_all(&uid).await {
        Ok(roles) => ApiResponse::success(roles, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
