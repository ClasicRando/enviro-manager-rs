use common::error::{EmError, EmResult};
use serde::{Deserialize, Serialize};
use sqlx::{Database, PgPool, Pool, Postgres};
use uuid::Uuid;

///
#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    uid: Uuid,
    full_name: String,
    roles: Vec<String>,
}

///
#[derive(Deserialize)]
pub struct CreateUserRequest {
    current_uid: Uuid,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
    roles: Vec<String>,
}

///
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    username: String,
    password: String,
    #[serde(flatten)]
    update_type: UpdateUserType,
}

impl UpdateUserRequest {
    pub fn username(&self) -> &str {
        &self.username
    }
}

///
#[derive(Deserialize)]
pub enum UpdateUserType {
    Username {
        new_username: String,
    },
    FullName {
        new_first_name: String,
        new_last_name: String,
    },
    ResetPassword {
        new_password: String,
    },
}

///
#[derive(Deserialize)]
pub struct ValidateUserRequest {
    username: String,
    password: String,
}

///
#[derive(Deserialize)]
pub struct ModifyUserRoleRequest {
    current_uid: Uuid,
    uid: Uuid,
    role: String,
    add: bool,
}

///
pub trait UserService: Clone + Send + Sync {
    type Database: Database;

    ///
    fn new(pool: &Pool<Self::Database>) -> Self;
    ///
    async fn create_user(&self, request: CreateUserRequest) -> EmResult<User>;
    ///
    async fn update(&self, request: &UpdateUserRequest) -> EmResult<User>;
    ///
    async fn validate_user(&self, request: ValidateUserRequest) -> EmResult<User>;
    ///
    async fn modify_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<User>;
}

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

    async fn create_user(&self, request: CreateUserRequest) -> EmResult<User> {
        let result = sqlx::query_as("select users.create_user($1, $2, $3, $4, $5, $6)")
            .bind(request.current_uid)
            .bind(&request.first_name)
            .bind(&request.last_name)
            .bind(&request.username)
            .bind(&request.password)
            .bind(&request.roles)
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

    async fn validate_user(&self, request: ValidateUserRequest) -> EmResult<User> {
        let result = sqlx::query_as("select users.validate_user($1, $2)")
            .bind(&request.username)
            .bind(&request.password)
            .fetch_optional(&self.pool)
            .await?;
        match result {
            Some(user) => Ok(user),
            None => Err(EmError::InvalidUser),
        }
    }

    async fn modify_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<User> {
        let query = if request.add {
            "call users.add_user_role($1, $2, $3)"
        } else {
            "call users.revoke_user_role($1, $2, $3)"
        };
        let user = sqlx::query_as(query)
            .bind(request.current_uid)
            .bind(request.uid)
            .bind(&request.role)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }
}
