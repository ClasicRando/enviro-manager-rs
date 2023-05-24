use common::{
    api::ApiRequestValidator,
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
    // pub(crate) current_uid: Uuid,
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

/// Default [ApiRequestValidator] for [CreateUserRequest]
pub struct CreateUserRequestValidator;

impl ApiRequestValidator for CreateUserRequestValidator {
    type Request = CreateUserRequest;

    fn validate(request: &Self::Request) -> EmResult<()> {
        if request.first_name.trim().is_empty() {
            Err((request, "first_name cannot be empty or whitespace"))?;
        }
        if request.last_name.trim().is_empty() {
            Err((request, "last_name cannot be empty or whitespace"))?;
        }
        if request.username.trim().is_empty() {
            Err((request, "username cannot be empty or whitespace"))?;
        }
        validate_password(&request.password)?;
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

/// Default [ApiRequestValidator] for [UpdateUserRequest]
pub struct UpdateUserRequestValidator;

impl ApiRequestValidator for UpdateUserRequestValidator {
    type Request = UpdateUserRequest;

    fn validate(request: &Self::Request) -> EmResult<()> {
        if request.validate_user.username.trim().is_empty() {
            Err((request, "username cannot be empty or whitespace"))?;
        }
        match &request.update_type {
            UpdateUserType::Username { new_username } => {
                if new_username.trim().is_empty() {
                    Err((request, "new_username cannot be empty or whitespace"))?;
                }
            }
            UpdateUserType::FullName {
                new_first_name,
                new_last_name,
            } => {
                if new_first_name.trim().is_empty() {
                    Err((request, "new_first_name cannot be empty or whitespace"))?;
                }
                if new_last_name.trim().is_empty() {
                    Err((request, "new_last_name cannot be empty or whitespace"))?;
                }
            }
            UpdateUserType::ResetPassword { new_password } => {
                validate_password(new_password)?;
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
    // pub(crate) current_uid: Uuid,
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
    type CreateRequestValidator: ApiRequestValidator<Request = CreateUserRequest>;
    type Database: Database;
    type UpdateRequestValidator: ApiRequestValidator<Request = UpdateUserRequest>;

    /// Create new instance of a [UserService]
    fn create(pool: &Pool<Self::Database>) -> Self;
    /// Create a new [User]. The user specified in `request` must have the 'admin' role to perform
    /// this action. Returns the newly created [User]
    async fn create_user(&self, current_uid: &Uuid, request: &CreateUserRequest) -> EmResult<User>;
    /// Read all [User]s from the database. The user specified as `current_uid` must have the
    /// 'admin' role to perform this action.
    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<User>>;
    /// Read a single [User] from the database
    async fn read_one(&self, uuid: &Uuid) -> EmResult<User>;
    /// Update the user specified within the `request`. Once the user is validated, the update type
    /// specified is performed and the new state of the [User] is returned.
    async fn update(&self, current_uid: &Uuid, request: &UpdateUserRequest) -> EmResult<User>;
    /// Validate that the specified user credentials match a user. If successful, return that [User]
    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User>;
    /// Modify a role for the user specified within the `request`. The action user specified in the
    /// `request` must have the 'add-role' role and is only able to add/revoke roles that they have
    /// themselves
    async fn modify_user_role(
        &self,
        current_uid: &Uuid,
        request: &ModifyUserRoleRequest,
    ) -> EmResult<User>;
}

#[cfg(test)]
pub(crate) mod test {
    use std::str::FromStr;

    use common::api::ApiRequestValidator;
    use rstest::rstest;

    use crate::service::{
        roles::RoleName,
        users::{
            validate_password, CreateUserRequest, CreateUserRequestValidator, UpdateUserRequest,
            UpdateUserRequestValidator, UpdateUserType, ValidateUserRequest,
        },
    };

    const VALID_PASSWORD: &str = "Va1idPa$$word";

    /// Utility method for creating a new [CreateUserRequest]
    pub(crate) fn create_user_request(
        first_name: &str,
        last_name: &str,
        username: &str,
        password: &str,
        roles: &[&str],
    ) -> CreateUserRequest {
        CreateUserRequest {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            roles: roles
                .iter()
                .map(|r| RoleName::from_str(r).unwrap())
                .collect(),
        }
    }

    /// Utility method for creating a new [ValidateUserRequest]
    pub(crate) fn validate_user_request(username: &str, password: &str) -> ValidateUserRequest {
        ValidateUserRequest {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    /// Utility method for creating a new [UpdateUserRequest]
    pub(crate) fn update_user_request(
        username: &str,
        password: &str,
        update_type: UpdateUserType,
    ) -> UpdateUserRequest {
        UpdateUserRequest {
            validate_user: validate_user_request(username, password),
            update_type,
        }
    }

    /// Utility method for creating a new [UpdateUserType::Username]
    pub(crate) fn update_username(new_username: &str) -> UpdateUserType {
        UpdateUserType::Username {
            new_username: new_username.to_string(),
        }
    }

    /// Utility method for creating a new [UpdateUserType::FullName]
    pub(crate) fn update_full_name(new_first_name: &str, new_last_name: &str) -> UpdateUserType {
        UpdateUserType::FullName {
            new_first_name: new_first_name.to_string(),
            new_last_name: new_last_name.to_string(),
        }
    }

    /// Utility method for creating a new [UpdateUserType::ResetPassword]
    pub(crate) fn reset_password(new_password: &str) -> UpdateUserType {
        UpdateUserType::ResetPassword {
            new_password: new_password.to_string(),
        }
    }

    #[rstest]
    #[case::valid_password(VALID_PASSWORD)]
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
        assert!(result.is_err());
    }

    #[rstest]
    #[case::valid_request(create_user_request("test", "test", "test", VALID_PASSWORD, &["admin"]))]
    fn create_user_request_should_validate_when(#[case] request: CreateUserRequest) {
        let result = CreateUserRequestValidator::validate(&request);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }

    #[rstest]
    #[case::invalid_password(create_user_request("test", "test", "test", "test", &["admin"]))]
    #[case::first_name_empty(create_user_request("", "test", "test", VALID_PASSWORD, &["admin"]))]
    #[case::last_name_empty(create_user_request("test", "", "test", VALID_PASSWORD, &["admin"]))]
    #[case::username_empty(create_user_request("test", "test", "", VALID_PASSWORD, &["admin"]))]
    fn create_user_request_should_fail_when(#[case] request: CreateUserRequest) {
        let result = CreateUserRequestValidator::validate(&request);
        assert!(result.is_err());
    }

    #[rstest]
    #[case::valid_update_username_request("test", VALID_PASSWORD, update_username("test"))]
    #[case::valid_update_full_name_request(
        "test",
        VALID_PASSWORD,
        update_full_name("test", "test")
    )]
    #[case::valid_reset_password_request("test", VALID_PASSWORD, reset_password(VALID_PASSWORD))]
    fn update_user_request_should_validate_when(
        #[case] username: &str,
        #[case] password: &str,
        #[case] update_type: UpdateUserType,
    ) {
        let request = update_user_request(username, password, update_type);
        let result = UpdateUserRequestValidator::validate(&request);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }

    #[rstest]
    #[case::new_username_empty("test", VALID_PASSWORD, update_username(""))]
    #[case::new_first_name_empty("test", VALID_PASSWORD, update_full_name("", "test"))]
    #[case::new_last_name_empty("test", VALID_PASSWORD, update_full_name("test", ""))]
    #[case::new_password_invalid("test", VALID_PASSWORD, reset_password(""))]
    fn update_user_request_should_fail_when(
        #[case] username: &str,
        #[case] password: &str,
        #[case] update_type: UpdateUserType,
    ) {
        let request = update_user_request(username, password, update_type);
        let result = UpdateUserRequestValidator::validate(&request);
        assert!(result.is_err());
    }
}
