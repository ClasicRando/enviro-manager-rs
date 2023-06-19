use common::error::{EmError, EmResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::role::{Role, RoleName};

/// User entity as the uuid of the user, their full name and all roles possessed by the user.
#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
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
    /// # Errors
    /// This function will return an error if the user does not have the `role` provided
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

    /// Returns a reference to the user's uid
    pub const fn uid(&self) -> &Uuid {
        &self.uid
    }

    /// Returns a string slice of the user's full name
    pub fn full_name(&self) -> &str {
        &self.full_name
    }
}
