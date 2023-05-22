use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Postgres};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use strum::{AsRefStr, EnumIter, EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::service::users::UserService;

/// EnviroManager user role
#[derive(Serialize)]
pub struct Role {
    /// Name of the role. Unique within all roles
    pub(crate) name: RoleName,
    /// Short description of the role
    pub(crate) description: &'static str,
}

impl<'q> Encode<'q, Postgres> for Role
    where
        &'q str: Encode<'q, Postgres>,
{
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let val = match self.name {
            RoleName::Admin => "admin",
            RoleName::AddRole => "add-role",
        };
        <&str as Encode<'q, Postgres>>::encode(val, buf)
    }

    fn size_hint(&self) -> usize {
        let val = match self.name {
            RoleName::Admin => "admin",
            RoleName::AddRole => "add-role",
        };
        <&str as Encode<'q, Postgres>>::size_hint(&val)
    }
}

/// All role names that exist as their common name
#[derive(Serialize, Deserialize, EnumIter, EnumString, IntoStaticStr, AsRefStr, PartialEq)]
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
    fn new(user_service: &Self::UserService) -> Self;
    /// Read all roles found in the database. Must be an admin user to access roles
    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<Role>>;
}
