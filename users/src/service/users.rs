use common::error::{EmError, EmResult};
use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool};
use uuid::Uuid;

use super::roles::Role;
use crate::service::roles::RoleName;

/// User entity as the uuid of the user, their full name and all roles possessed by the user.
#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    /// Unique identifier of the user
    pub(crate) uid: Uuid,
    /// First name and last name of the user in a single string
    pub(crate) full_name: String,
    /// Collection of roles the user possesses
    pub(crate) roles: Vec<Role>,
}

impl User {
    /// Checks the current roles of the [User] against the `role` name provided. If any of the roles
    /// match or the user is an admin, return [Ok]. Otherwise, return an [EmError::MissingPrivilege]
    /// error.
    pub fn check_role(&self, role: RoleName) -> EmResult<()> {
        if self
            .roles
            .iter()
            .any(|r| r.name == role.as_str() || r.name == "admin")
        {
            return Ok(());
        }
        Err(EmError::MissingPrivilege {
            role: role.as_str().to_string(),
            uid: self.uid,
        })
    }
}

/// Request object for creating a new user
#[derive(Deserialize)]
pub struct CreateUserRequest {
    /// uuid of the user attempting to perform the action
    pub(crate) current_uid: Uuid,
    /// First name of the user to be created
    pub(crate) first_name: String,
    /// Last name of the user to be created
    pub(crate) last_name: String,
    /// Username of the user to be created. Must be unique for all users
    pub(crate) username: String,
    /// Password of the user to be created. Must follow rules specified here
    pub(crate) password: String,
    /// Roles of the user to be created
    pub(crate) roles: Vec<String>,
}

/// Request object for updating an existing user
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    /// Username of the user to updated. Required to verify user before updating.
    #[serde(flatten)]
    pub(crate) validate_user: ValidateUserRequest,
    /// Update variation the is required to be performed
    #[serde(flatten)]
    pub(crate) update_type: UpdateUserType,
}

impl UpdateUserRequest {
    /// Get the username of the update request as a string slice
    pub fn username(&self) -> &str {
        &self.validate_user.username
    }
}

/// User update type variations
#[derive(Deserialize)]
#[serde(untagged)]
pub enum UpdateUserType {
    /// User is attempting to update the user's username to a new value
    Username { new_username: String },
    /// User is attempting to update the user's first and last name
    FullName {
        new_first_name: String,
        new_last_name: String,
    },
    /// User is attempting to update the user's password to a new value
    ResetPassword { new_password: String },
}

/// Request object to validate the user given their username and password
#[derive(Deserialize)]
pub struct ValidateUserRequest {
    /// Username of the user to verify it's credentials
    pub(crate) username: String,
    /// Password of the user to verify it's credentials
    pub(crate) password: String,
}

/// Request object to allow an admin user to add or revoke another users role
#[derive(Deserialize)]
pub struct ModifyUserRoleRequest {
    /// uuid of the user attempting to perform the action
    pub(crate) current_uid: Uuid,
    /// uuid of the user to perform the action on
    pub(crate) uid: Uuid,
    /// Name of the role to modify for the specified `uid`
    pub(crate) role: String,
    /// Flag indicating if the role should be added or revoked for the specified `uid`
    pub(crate) add: bool,
}

/// Service for interacting with the user system. Allows for reading users as well as creating new
/// and modifying existing users.
pub trait UserService
where
    Self: Clone + Send + Sync,
{
    type Database: Database;

    /// Create new instance of a [UserService]
    fn new(pool: &Pool<Self::Database>) -> Self;
    /// Create a new [User]. The user specified in `request` must have the 'admin' role to perform
    /// this action. Returns the newly created [User]
    async fn create_user(&self, request: &CreateUserRequest) -> EmResult<User>;
    /// Read all [User]s from the database. The user specified as `current_uid` must have the
    /// 'admin' role to perform this action.
    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<User>>;
    /// Read a single [User] from the database
    async fn read_one(&self, uuid: &Uuid) -> EmResult<User>;
    /// Update the user specified within the `request`. Once the user is validated, the update type
    /// specified is performed and the new state of the [User] is returned.
    async fn update(&self, request: &UpdateUserRequest) -> EmResult<User>;
    /// Validate that the specified user credentials match a user. If successful, return that [User]
    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User>;
    /// Modify a role for the user specified within the `request`. The action user specified in the
    /// `request` must have the 'add-role' role and is only able to add/revoke roles that they have
    /// themselves
    async fn modify_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<User>;
}
