use common::error::{EmError, EmResult};
use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool};
use uuid::Uuid;
use crate::service::roles::RoleName;

use super::roles::Role;

///
#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub(crate) uid: Uuid,
    pub(crate) full_name: String,
    pub(crate) roles: Vec<Role>,
}

impl User {
    /// Checks the current roles of the [User] against the `role` name provided. If any of the roles
    /// match or the user is an admin, return [Ok]. Otherwise, return an [EmError::MissingPrivilege]
    /// error.
    pub fn check_role(&self, role: RoleName) -> EmResult<()> {
        if self.roles.iter().any(|r| r.name == role.as_str() || r.name == "admin") {
            return Ok(())
        }
        Err(EmError::MissingPrivilege { role: role.as_str().to_string(), uid: self.uid })
    }
}

///
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub(crate) current_uid: Uuid,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) roles: Vec<String>,
}

///
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub(crate) username: String,
    pub(crate) password: String,
    #[serde(flatten)]
    pub(crate) update_type: UpdateUserType,
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
    pub(crate) username: String,
    pub(crate) password: String,
}

///
#[derive(Deserialize)]
pub struct ModifyUserRoleRequest {
    pub(crate) current_uid: Uuid,
    pub(crate) uid: Uuid,
    pub(crate) role: String,
    pub(crate) add: bool,
}

///
pub trait UserService: Clone + Send + Sync {
    type Database: Database;

    ///
    fn new(pool: &Pool<Self::Database>) -> Self;
    ///
    async fn create_user(&self, request: &CreateUserRequest) -> EmResult<User>;
    ///
    async fn get_user(&self, uuid: Uuid) -> EmResult<User>;
    ///
    async fn update(&self, request: &UpdateUserRequest) -> EmResult<User>;
    ///
    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User>;
    ///
    async fn modify_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<User>;
}
