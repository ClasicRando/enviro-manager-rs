use common::api::ApiResponse;
use log::error;

use crate::service::roles::{CreateRoleRequest, Role, RoleService, UpdateRoleRequest};

/// API endpoint to fetch all roles
pub async fn roles<R>(service: actix_web::web::Data<R>) -> ApiResponse<Vec<Role>>
where
    R: RoleService,
{
    match service.read_all().await {
        Ok(roles) => ApiResponse::success(roles),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to create a new role
pub async fn create_role<R>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<R>,
) -> ApiResponse<()>
where
    R: RoleService,
{
    let role_request: CreateRoleRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize role creation request. Error: {}",
                error
            ));
        }
    };
    match service.create_role(&role_request).await {
        Ok(_) => ApiResponse::message(format!("Successfully created role `{}`", role_request.name)),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to update an existing role
pub async fn update_role<R>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<R>,
) -> ApiResponse<()>
where
    R: RoleService,
{
    let role_request: UpdateRoleRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize role update request. Error: {}",
                error
            ));
        }
    };
    match service.update_role(&role_request).await {
        Ok(_) => ApiResponse::message(format!("Successfully update role `{}`", role_request.name)),
        Err(error) => ApiResponse::error(error),
    }
}
