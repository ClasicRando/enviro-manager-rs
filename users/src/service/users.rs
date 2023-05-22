use common::{
    api::ApiRequest,
    error::{EmError, EmResult},
};
use lazy_regex::regex;
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
            .any(|r| r.name == role || r.name == RoleName::Admin)
        {
            return Ok(());
        }
        Err(EmError::MissingPrivilege {
            role: role.into(),
            uid: self.uid,
        })
    }
}

/// Validate that the provided `password` meets the rules prescribed for password
fn validate_password(password: &str) -> EmResult<()> {
    if password.trim().is_empty() {
        return Err(EmError::InvalidPassword {
            reason: "Must not be an empty string or whitespace",
        });
    }
    if !regex!("[A-Z]").is_match(password) {
        return Err(EmError::InvalidPassword {
            reason: "Must contain at least 1 uppercase character",
        });
    }
    if !regex!(r"\d").is_match(password) {
        return Err(EmError::InvalidPassword {
            reason: "Must contain at least 1 digit character",
        });
    }
    if !regex!(r"\W").is_match(password) {
        return Err(EmError::InvalidPassword {
            reason: "Must contain at least 1 non-alphanumeric character",
        });
    }
    Ok(())
}

/// Request object for creating a new user
#[derive(Deserialize, Debug)]
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
    pub(crate) roles: Vec<RoleName>,
}

impl ApiRequest for CreateUserRequest {
    fn validate(&self) -> EmResult<()> {
        if self.first_name.trim().is_empty() {
            return Err((self, "first_name cannot be empty or whitespace").into());
        }
        if self.last_name.trim().is_empty() {
            return Err((self, "last_name cannot be empty or whitespace").into());
        }
        if self.username.trim().is_empty() {
            return Err((self, "username cannot be empty or whitespace").into());
        }
        if self.password.trim().is_empty() {
            return Err((self, "password cannot be empty or whitespace").into());
        }
        validate_password(&self.password)?;
        Ok(())
    }
}

/// Request object for updating an existing user
#[derive(Deserialize, Debug)]
pub struct UpdateUserRequest {
    /// Username of the user to updated. Required to verify user before updating.
    #[serde(flatten)]
    pub(crate) validate_user: ValidateUserRequest,
    /// Update variation the is required to be performed
    #[serde(flatten)]
    pub(crate) update_type: UpdateUserType,
}

impl ApiRequest for UpdateUserRequest {
    fn validate(&self) -> EmResult<()> {
        match &self.update_type {
            UpdateUserType::Username { new_username } => {
                if new_username.trim().is_empty() {
                    return Err((self, "new_username cannot be empty or whitespace").into());
                }
            }
            UpdateUserType::FullName {
                new_first_name,
                new_last_name,
            } => {
                if new_first_name.trim().is_empty() {
                    return Err((self, "new_first_name cannot be empty or whitespace").into());
                }
                if new_last_name.trim().is_empty() {
                    return Err((self, "new_last_name cannot be empty or whitespace").into());
                }
            }
            UpdateUserType::ResetPassword { new_password } => {
                if new_password.trim().is_empty() {
                    return Err((self, "new_password cannot be empty or whitespace").into());
                }
            }
        }
        Ok(())
    }
}

impl UpdateUserRequest {
    /// Get the username of the update request as a string slice
    pub fn username(&self) -> &str {
        &self.validate_user.username
    }
}

/// User update type variations
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
pub struct ValidateUserRequest {
    /// Username of the user to verify it's credentials
    pub(crate) username: String,
    /// Password of the user to verify it's credentials
    pub(crate) password: String,
}

/// Request object to allow an admin user to add or revoke another users role
#[derive(Deserialize, Debug)]
pub struct ModifyUserRoleRequest {
    /// uuid of the user attempting to perform the action
    pub(crate) current_uid: Uuid,
    /// uuid of the user to perform the action on
    pub(crate) uid: Uuid,
    /// Name of the role to modify for the specified `uid`
    pub(crate) role: RoleName,
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

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::{CreateUserRequest, ModifyUserRoleRequest, UpdateUserRequest, validate_password};

    #[rstest]
    #[case::valid_password("Va1idPa$$word")]
    fn validate_password_should_succeed_when(#[case] password: &str) {
        let result = validate_password(password);

        assert!(result.is_ok())
    }

    #[rstest]
    #[case::empty("")]
    #[case::whitespace1(" ")]
    #[case::whitespace2("\t ")]
    #[case::whitespace3(" \n ")]
    #[case::whitespace4("\t \n ")]
    #[case::missing_uppercase("test")]
    #[case::missing_digit("Test")]
    #[case::missing_non_alphanumeric("Test1")]
    fn validate_password_should_fail_when(#[case] password: &str) {
        let result = validate_password(password);

        assert!(result.is_err())
    }
}
