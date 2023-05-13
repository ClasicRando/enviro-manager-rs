use common::{api::ApiResponse, error::EmError};
use log::error;

use crate::service::users::{
    CreateUserRequest, ModifyUserRoleRequest, UpdateUserRequest, User, UserService,
    ValidateUserRequest,
};

/// API endpoint to create a new user
pub async fn create_user<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let user_request: CreateUserRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize user creation request. Error: {}",
                error
            ));
        }
    };
    match service.create_user(&user_request).await {
        Ok(user) => ApiResponse::success(user),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to update a user
pub async fn update_user<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<()>
where
    U: UserService,
{
    let user_request: UpdateUserRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize an update user request. Error: {}",
                error
            ));
        }
    };
    match service.update(&user_request).await {
        Ok(_) => ApiResponse::message(format!("Updated username for {}", user_request.username())),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to validate a users credentials. If successful, a [User] instance is returned
pub async fn validate_user<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let user_request: ValidateUserRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize user validation request. Error: {}",
                error
            ));
        }
    };
    match service.validate_user(&user_request).await {
        Ok(user) => ApiResponse::success(user),
        Err(error) if matches!(error, EmError::InvalidUser) => {
            ApiResponse::failure("Invalid user credentials".to_string())
        }
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to add/remove a role for a specified user
pub async fn modify_user_role<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let user_request: ModifyUserRoleRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize full name update request. Error: {}",
                error
            ));
        }
    };
    match service.modify_user_role(&user_request).await {
        Ok(user) => ApiResponse::success(user),
        Err(error) => ApiResponse::error(error),
    }
}
