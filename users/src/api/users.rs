use common::{
    api::{request::ApiRequest, ApiResponse, QueryApiFormat},
    error::EmError,
};
use log::error;

use super::UidRequest;
use crate::service::users::{
    CreateUserRequest, ModifyUserRoleRequest, UpdateUserRequest, User, UserService,
    ValidateUserRequest,
};

/// API endpoint to create a new user from a MessagePack body
pub async fn create_user<U>(
    api_request: ApiRequest<CreateUserRequest>,
    service: actix_web::web::Data<U>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let format = query.into_inner();
    let user_request = api_request.into_inner();
    match service.create_user(&user_request).await {
        Ok(user) => ApiResponse::success(user, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to read all users
pub async fn read_users<U>(
    api_request: ApiRequest<UidRequest>,
    service: actix_web::web::Data<U>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<Vec<User>>
where
    U: UserService,
{
    let format = query.into_inner();
    let UidRequest { uid } = api_request.into_inner();
    match service.read_all(&uid).await {
        Ok(user) => ApiResponse::success(user, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to update a user
pub async fn update_user<U>(
    api_request: ApiRequest<UpdateUserRequest>,
    service: actix_web::web::Data<U>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<()>
where
    U: UserService,
{
    let format = query.into_inner();
    let user_request = api_request.into_inner();
    match service.update(&user_request).await {
        Ok(user) => ApiResponse::message(format!("Updated user {}", user.uid), format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}

/// API endpoint to validate a users credentials. If successful, a [User] instance is returned
pub async fn validate_user<U>(
    api_request: ApiRequest<ValidateUserRequest>,
    service: actix_web::web::Data<U>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let query = query.into_inner();
    let user_request = api_request.into_inner();
    match service.validate_user(&user_request).await {
        Ok(user) => ApiResponse::success(user, query.f),
        Err(error) if matches!(error, EmError::InvalidUser) => {
            ApiResponse::failure("Invalid user credentials", query.f)
        }
        Err(error) => {
            error!("{error}");
            ApiResponse::error(error, query.f)
        }
    }
}

/// API endpoint to add/remove a role for a specified user
pub async fn modify_user_role<U>(
    api_request: ApiRequest<ModifyUserRoleRequest>,
    service: actix_web::web::Data<U>,
    query: actix_web::web::Query<QueryApiFormat>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let format = query.into_inner();
    let user_request = api_request.into_inner();
    match service.modify_user_role(&user_request).await {
        Ok(user) => ApiResponse::success(user, format.f),
        Err(error) => ApiResponse::error(error, format.f),
    }
}
