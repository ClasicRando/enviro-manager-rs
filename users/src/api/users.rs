use actix_session::Session;
use common::{
    api::{request::ApiRequest, validate_session, ApiContentFormat, ApiResponse, QueryApiFormat},
    error::EmError,
};
use log::error;

use crate::service::users::{
    CreateUserRequest, ModifyUserRoleRequest, UpdateUserRequest, User, UserService,
    ValidateUserRequest,
};

/// API endpoint to create a new user from a MessagePack body
pub async fn create_user<U>(
    session: Session,
    api_request: ApiRequest<CreateUserRequest>,
    service: actix_web::web::Data<U>,
    path: actix_web::web::Path<ApiContentFormat>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let format = path.into_inner();
    let uuid = match validate_session(&session, format) {
        Ok(inner) => inner,
        Err(response) => return response,
    };
    let user_request = api_request.into_inner();
    match service.create_user(&uuid, &user_request).await {
        Ok(user) => ApiResponse::success(user, format),
        Err(error) => ApiResponse::error(error, format),
    }
}

/// API endpoint to read all users
pub async fn read_users<U>(
    session: Session,
    service: actix_web::web::Data<U>,
    path: actix_web::web::Path<ApiContentFormat>,
) -> ApiResponse<Vec<User>>
where
    U: UserService,
{
    let format = path.into_inner();
    let uuid = match validate_session(&session, format) {
        Ok(inner) => inner,
        Err(response) => return response,
    };
    match service.read_all(&uuid).await {
        Ok(user) => ApiResponse::success(user, format),
        Err(error) => ApiResponse::error(error, format),
    }
}

/// API endpoint to update a user
pub async fn update_user<U>(
    session: Session,
    api_request: ApiRequest<UpdateUserRequest>,
    service: actix_web::web::Data<U>,
    path: actix_web::web::Path<ApiContentFormat>,
) -> ApiResponse<()>
where
    U: UserService,
{
    let format = path.into_inner();
    let uuid = match validate_session(&session, format) {
        Ok(inner) => inner,
        Err(response) => return response,
    };
    let user_request = api_request.into_inner();
    match service.update(&uuid, &user_request).await {
        Ok(user) => ApiResponse::message(format!("Updated user {}", user.uid), format),
        Err(error) => ApiResponse::error(error, format),
    }
}

/// API endpoint to validate a users credentials. If successful, a [User] instance is returned
pub async fn validate_user<U>(
    session: Session,
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
        Ok(user) => {
            if let Err(error) = session.insert("em_uid", user.uid) {
                error!("{error}");
                return ApiResponse::error(error.into(), query.f);
            }
            session.renew();
            ApiResponse::success(user, query.f)
        }
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
    session: Session,
    api_request: ApiRequest<ModifyUserRoleRequest>,
    service: actix_web::web::Data<U>,
    path: actix_web::web::Path<ApiContentFormat>,
) -> ApiResponse<User>
where
    U: UserService,
{
    let format = path.into_inner();
    let uuid = match validate_session(&session, format) {
        Ok(inner) => inner,
        Err(response) => return response,
    };
    let user_request = api_request.into_inner();
    match service.modify_user_role(&uuid, &user_request).await {
        Ok(user) => ApiResponse::success(user, format),
        Err(error) => ApiResponse::error(error, format),
    }
}
