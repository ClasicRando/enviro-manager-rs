use std::fmt::Display;

use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Database, PgPool, Pool, Postgres};

///
#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(transparent)]
pub struct EmUid(i64);

impl Display for EmUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User ID: {}", self.0)
    }
}

///
#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    em_uid: EmUid,
    full_name: String,
    roles: Vec<String>,
}

///
#[derive(Deserialize)]
pub struct UserRequest {
    current_em_uid: EmUid,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
    roles: Vec<String>,
}

///
#[derive(Deserialize)]
pub struct UsernameUpdateRequest {
    username: String,
    password: String,
    new_username: String,
}

impl UsernameUpdateRequest {
    ///
    pub fn username(&self) -> &str {
        &self.username
    }
}

///
#[derive(Deserialize)]
pub struct FullNameUpdateRequest {
    username: String,
    password: String,
    new_first_name: String,
    new_last_name: String,
}

impl FullNameUpdateRequest {
    ///
    pub fn username(&self) -> &str {
        &self.username
    }
}

///
#[derive(Deserialize)]
pub struct ValidateUserRequest {
    username: String,
    password: String,
}

///
#[derive(Deserialize)]
pub struct ResetUserPasswordRequest {
    username: String,
    password: String,
    new_password: String,
}

impl ResetUserPasswordRequest {
    ///
    pub fn username(&self) -> &str {
        &self.username
    }
}

///
#[derive(Deserialize)]
pub struct ModifyUserRoleRequest {
    current_em_uid: EmUid,
    em_uid: EmUid,
    role: String,
}

impl ModifyUserRoleRequest {
    ///
    pub fn em_uid(&self) -> &EmUid {
        &self.em_uid
    }

    ///
    pub fn role(&self) -> &str {
        &self.role
    }
}

///
#[async_trait::async_trait]
pub trait UserService: Clone + Send + Sync {
    type Database: Database;

    ///
    fn new(pool: &Pool<Self::Database>) -> Self;
    ///
    async fn create_user(&self, request: UserRequest) -> EmResult<User>;
    ///
    async fn update_username(&self, request: &UsernameUpdateRequest) -> EmResult<()>;
    ///
    async fn update_full_name(&self, request: &FullNameUpdateRequest) -> EmResult<()>;
    ///
    async fn validate_user(&self, request: ValidateUserRequest) -> EmResult<Option<User>>;
    ///
    async fn reset_password(&self, request: &ResetUserPasswordRequest) -> EmResult<()>;
    ///
    async fn add_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<()>;
    ///
    async fn revoke_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<()>;
}

/// Postgresql implementation of [UserService]
#[derive(Clone)]
pub struct PgUserService {
    pool: PgPool,
}

#[async_trait::async_trait]
impl UserService for PgUserService {
    type Database = Postgres;

    fn new(pool: &Pool<Self::Database>) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create_user(&self, request: UserRequest) -> EmResult<User> {
        let result =
            sqlx::query_as("select users.create_user($1, $2, $3, $4, $5, $6)")
                .bind(&request.current_em_uid)
                .bind(&request.first_name)
                .bind(&request.last_name)
                .bind(&request.username)
                .bind(&request.password)
                .bind(&request.roles)
                .fetch_one(&self.pool)
                .await?;
        Ok(result)
    }

    async fn update_username(&self, request: &UsernameUpdateRequest) -> EmResult<()> {
        sqlx::query("call users.update_username($1, $2, $3)")
            .bind(&request.username)
            .bind(&request.password)
            .bind(&request.new_username)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_full_name(&self, request: &FullNameUpdateRequest) -> EmResult<()> {
        sqlx::query("call users.update_full_name($1, $2, $3, $4)")
            .bind(&request.username)
            .bind(&request.password)
            .bind(&request.new_first_name)
            .bind(&request.new_last_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn validate_user(&self, request: ValidateUserRequest) -> EmResult<Option<User>> {
        let result = sqlx::query_as("select users.validate_user($1, $2)")
            .bind(&request.username)
            .bind(&request.password)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    async fn reset_password(&self, request: &ResetUserPasswordRequest) -> EmResult<()> {
        sqlx::query("call users.reset_password($1, $2, $3)")
            .bind(&request.username)
            .bind(&request.password)
            .bind(&request.new_password)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn add_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<()> {
        sqlx::query("call users.add_user_role($1, $2, $3)")
            .bind(&request.current_em_uid)
            .bind(&request.em_uid)
            .bind(&request.role)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<()> {
        sqlx::query("call users.revoke_user_role($1, $2, $3)")
            .bind(&request.current_em_uid)
            .bind(&request.em_uid)
            .bind(&request.role)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
