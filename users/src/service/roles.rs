use common::error::EmResult;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::service::users::UserService;

/// EnviroManager user role
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Role {
    /// Name of the role. Unique within all roles
    pub(crate) name: RoleName,
    /// Short description of the role
    pub(crate) description: String,
}

/// All role names that exist as their common name
#[derive(
    Serialize,
    Deserialize,
    EnumIter,
    EnumString,
    IntoStaticStr,
    AsRefStr,
    PartialEq,
    Debug,
    Copy,
    Clone,
)]
pub enum RoleName {
    #[serde(rename = "admin")]
    #[strum(serialize = "admin")]
    Admin,
    #[serde(rename = "add-role")]
    #[strum(serialize = "add-role")]
    AddRole,
}

impl RoleName {
    /// Gets the string representation of the [RoleName] as seen in the database
    pub const fn description(&self) -> &'static str {
        match self {
            RoleName::Admin => "Role with full access to all other roles",
            RoleName::AddRole => {
                "Provides a user with the ability to add/remove roles from a user. However, this \
                 is limited to the roles of the current user."
            }
        }
    }
}

/// Service for interacting with the role system. Allows for reading all roles as well as creating
/// new and modifying existing roles. Requires the [UserService] as an associated type to fetch
/// user data to confirm the roles of a user before creating/modifying roles.
pub trait RoleService
where
    Self: Clone + Send + Sync,
{
    type UserService: UserService;

    /// Create new instance of a [RoleService]. Both parameters are references to allow for cloning
    /// of the value.
    fn create(user_service: &Self::UserService) -> Self;
    /// Read all roles found in the database. Must be an admin user to access roles
    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<Role>>;
}
