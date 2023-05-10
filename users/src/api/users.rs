use common::api::ApiResponse;
use log::error;

use crate::services::users::{
    FullNameUpdateRequest, ModifyUserRoleRequest, ResetUserPasswordRequest, User, UserRequest,
    UserService, UsernameUpdateRequest, ValidateUserRequest,
};

/// API endpoint to create a new user
pub async fn create_user<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let user_request: UserRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize user creation request. Error: {}",
                error
            ));
        }
    };
    match service.create_user(user_request).await {
        Ok(user) => ApiResponse::success(user),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to update a user's username
pub async fn update_username<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<()>
where
    U: UserService,
{
    let user_request: UsernameUpdateRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize username update request. Error: {}",
                error
            ));
        }
    };
    match service.update_username(&user_request).await {
        Ok(_) => ApiResponse::message(format!("Updated username for {}", user_request.username())),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to update a user's full name
pub async fn update_full_name<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<()>
where
    U: UserService,
{
    let user_request: FullNameUpdateRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize full name update request. Error: {}",
                error
            ));
        }
    };
    match service.update_full_name(&user_request).await {
        Ok(_) => ApiResponse::message(format!("Updated full name for {}", user_request.username())),
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
    match service.validate_user(user_request).await {
        Ok(Some(user)) => ApiResponse::success(user),
        Ok(None) => ApiResponse::failure("Invalid user credentials".to_string()),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to reset a user's password
pub async fn reset_password<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<()>
where
    U: UserService,
{
    let user_request: ResetUserPasswordRequest = match rmp_serde::from_slice(&data) {
        Ok(inner) => inner,
        Err(error) => {
            error!("{}", error);
            return ApiResponse::failure(format!(
                "Could no deserialize full name update request. Error: {}",
                error
            ));
        }
    };
    match service.reset_password(&user_request).await {
        Ok(_) => ApiResponse::message(format!("Updated password for {}", user_request.username())),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to add a new role to a user
pub async fn add_user_role<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<()>
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
    match service.add_user_role(&user_request).await {
        Ok(_) => ApiResponse::message(format!(
            "Added role `{}` for {}",
            user_request.role(),
            user_request.em_uid()
        )),
        Err(error) => ApiResponse::error(error),
    }
}

/// API endpoint to remove a role from a user
pub async fn revoke_user_role<U>(
    data: actix_web::web::Bytes,
    service: actix_web::web::Data<U>,
) -> ApiResponse<()>
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
    match service.revoke_user_role(&user_request).await {
        Ok(_) => ApiResponse::message(format!(
            "Removed role `{}` from {}",
            user_request.role(),
            user_request.em_uid()
        )),
        Err(error) => ApiResponse::error(error),
    }
}
