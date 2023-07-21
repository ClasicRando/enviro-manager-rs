use common::{
    api::ApiRequestValidator,
    database::Database,
    error::{EmError, EmResult},
};
use lazy_regex::regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::{role::RoleName, user::User};

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
    /// Full name of the user to be created
    pub(crate) full_name: String,
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
    type ErrorMessage = String;
    type Request = CreateUserRequest;

    fn validate(request: &Self::Request) -> Result<(), Self::ErrorMessage> {
        if request.full_name.trim().is_empty() {
            Err("full_name cannot be empty or whitespace")?;
        }
        if request.username.trim().is_empty() {
            Err("username cannot be empty or whitespace")?;
        }
        if let Err(error) = validate_password(&request.password) {
            Err(format!("{error}"))?
        }
        Ok(())
    }
}

/// Request object for updating an existing user
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserRequest {
    /// uuid of the user attempting to perform the action
    pub(crate) update_uid: Uuid,
    /// User is attempting to update the user's username to a new value if Some
    pub(crate) new_username: Option<String>,
    /// User is attempting to update the user's full name if Some
    pub(crate) new_name: Option<String>,
}

impl UpdateUserRequest {
    /// Create a new [UpdateUserRequest] as the 2 components of a request
    pub const fn new(
        update_uid: Uuid,
        new_username: Option<String>,
        new_name: Option<String>,
    ) -> Self {
        Self {
            update_uid,
            new_username,
            new_name,
        }
    }
}

/// Default [ApiRequestValidator] for [UpdateUserRequest]
pub struct UpdateUserRequestValidator;

impl ApiRequestValidator for UpdateUserRequestValidator {
    type ErrorMessage = String;
    type Request = UpdateUserRequest;

    fn validate(request: &Self::Request) -> Result<(), Self::ErrorMessage> {
        if let Some(new_username) = &request.new_username {
            if new_username.trim().is_empty() {
                Err("new_username cannot be empty or whitespace")?;
            }
        }
        if let Some(new_name) = &request.new_name {
            if new_name.trim().is_empty() {
                Err("new_name cannot be empty or whitespace")?;
            }
        }
        Ok(())
    }
}

/// Request object to validate the user given their username and password
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidateUserRequest {
    /// Username of the user to verify it's credentials
    pub(crate) username: String,
    /// Password of the user to verify it's credentials
    pub(crate) password: String,
}

impl ValidateUserRequest {
    /// Create a new [ValidateUserRequest] as `username` and `password` values that can be converted
    /// to strings
    pub fn new<S: Into<String>>(username: S, password: S) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
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
#[allow(clippy::unwrap_used)]
pub(crate) mod test {
    use std::str::FromStr;

    use common::api::ApiRequestValidator;
    use rstest::rstest;
    use uuid::Uuid;

    use crate::{
        data::role::RoleName,
        service::users::{
            validate_password, CreateUserRequest, CreateUserRequestValidator, UpdateUserRequest,
            UpdateUserRequestValidator, ValidateUserRequest,
        },
    };

    const VALID_PASSWORD: &str = "Va1idPa$$word";

    /// Utility method for creating a new [CreateUserRequest]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_user_request(
        full_name: &str,
        username: &str,
        password: &str,
        roles: &[&str],
    ) -> CreateUserRequest {
        CreateUserRequest {
            full_name: full_name.to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
            roles: roles
                .iter()
                .map(|r| RoleName::from_str(r).unwrap())
                .collect(),
        }
    }

    /// Utility method for creating a new [ValidateUserRequest]
    pub(crate) fn validate_user_request(username: &str, password: &str) -> ValidateUserRequest {
        ValidateUserRequest {
            username: username.to_owned(),
            password: password.to_owned(),
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
    #[case::valid_request(create_user_request("test", "test", VALID_PASSWORD, &["admin"]))]
    fn create_user_request_should_validate_when(#[case] request: CreateUserRequest) {
        let result = CreateUserRequestValidator::validate(&request);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }

    #[rstest]
    #[case::invalid_password(create_user_request("test", "test", "test", &["admin"]))]
    #[case::name_empty(create_user_request("", "test", VALID_PASSWORD, &["admin"]))]
    #[case::username_empty(create_user_request("test", "", VALID_PASSWORD, &["admin"]))]
    fn create_user_request_should_fail_when(#[case] request: CreateUserRequest) {
        let result = CreateUserRequestValidator::validate(&request);
        assert!(result.is_err());
    }

    #[rstest]
    #[case::valid_update_username_request(Uuid::new_v4(), Some(String::from("test")), None)]
    #[case::valid_update_full_name_request(Uuid::new_v4(), None, Some(String::from("test")))]
    #[case::valid_update_all_request(
        Uuid::new_v4(),
        Some(String::from("test")),
        Some(String::from("test"))
    )]
    fn update_user_request_should_validate_when(
        #[case] uid: Uuid,
        #[case] new_name: Option<String>,
        #[case] new_username: Option<String>,
    ) {
        let request = UpdateUserRequest::new(uid, new_name, new_username);
        let result = UpdateUserRequestValidator::validate(&request);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }

    #[rstest]
    #[case::new_username_empty(Uuid::new_v4(), Some(String::new()), None)]
    #[case::new_name_empty(Uuid::new_v4(), None, Some(String::new()))]
    fn update_user_request_should_fail_when(
        #[case] uid: Uuid,
        #[case] new_name: Option<String>,
        #[case] new_username: Option<String>,
    ) {
        let request = UpdateUserRequest::new(uid, new_name, new_username);
        let result = UpdateUserRequestValidator::validate(&request);
        assert!(result.is_err());
    }
}
