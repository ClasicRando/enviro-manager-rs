use common::error::{EmError, EmResult};
use sqlx::{PgPool, Pool, Postgres};

use crate::service::users::{
    CreateUserRequest, ModifyUserRoleRequest, UpdateUserRequest, UpdateUserType, User, UserService,
    ValidateUserRequest,
};

/// Postgresql implementation of [UserService]
#[derive(Clone)]
pub struct PgUserService {
    pool: PgPool,
}

impl PgUserService {
    async fn update_full_name(
        &self,
        username: &str,
        password: &str,
        new_first_name: &str,
        new_last_name: &str,
    ) -> EmResult<User> {
        let user = sqlx::query_as("call users.update_full_name($1, $2, $3, $4)")
            .bind(username)
            .bind(password)
            .bind(new_first_name)
            .bind(new_last_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    async fn update_username(
        &self,
        username: &str,
        password: &str,
        new_username: &str,
    ) -> EmResult<User> {
        let user = sqlx::query_as("call users.update_username($1, $2, $3)")
            .bind(username)
            .bind(password)
            .bind(new_username)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    async fn reset_password(
        &self,
        username: &str,
        password: &str,
        new_password: &str,
    ) -> EmResult<User> {
        let user = sqlx::query_as("call users.reset_password($1, $2, $3)")
            .bind(username)
            .bind(password)
            .bind(new_password)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }
}

impl UserService for PgUserService {
    type Database = Postgres;

    fn new(pool: &Pool<Self::Database>) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create_user(&self, request: &CreateUserRequest) -> EmResult<User> {
        let CreateUserRequest {
            current_uid,
            first_name,
            last_name,
            username,
            password,
            roles,
        } = request;
        let result = sqlx::query_as("select users.create_user($1, $2, $3, $4, $5, $6)")
            .bind(current_uid)
            .bind(first_name)
            .bind(last_name)
            .bind(username)
            .bind(password)
            .bind(roles)
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }

    async fn update(&self, request: &UpdateUserRequest) -> EmResult<User> {
        let UpdateUserRequest {
            username,
            password,
            update_type,
        } = request;
        match update_type {
            UpdateUserType::Username { new_username } => {
                self.update_username(username, password, new_username).await
            }
            UpdateUserType::FullName {
                new_first_name,
                new_last_name,
            } => {
                self.update_full_name(username, password, new_first_name, new_last_name)
                    .await
            }
            UpdateUserType::ResetPassword { new_password } => {
                self.reset_password(username, password, new_password).await
            }
        }
    }

    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User> {
        let ValidateUserRequest { username, password } = request;
        let result = sqlx::query_as("select users.validate_user($1, $2)")
            .bind(username)
            .bind(password)
            .fetch_optional(&self.pool)
            .await?;
        match result {
            Some(user) => Ok(user),
            None => Err(EmError::InvalidUser),
        }
    }

    async fn modify_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<User> {
        let ModifyUserRoleRequest {
            current_uid,
            uid,
            role,
            add,
        } = request;
        let query = if *add {
            "call users.add_user_role($1, $2, $3)"
        } else {
            "call users.revoke_user_role($1, $2, $3)"
        };
        let user = sqlx::query_as(query)
            .bind(current_uid)
            .bind(uid)
            .bind(role)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }
}
