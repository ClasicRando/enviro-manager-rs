use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool};
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
    async fn update(&self, request: &UpdateUserRequest) -> EmResult<User>;
    ///
    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User>;
    ///
    async fn modify_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<User>;
}
