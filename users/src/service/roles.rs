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
    #[serde(rename = "check")]
    #[strum(serialize = "check")]
    Check,
    #[serde(rename = "collection")]
    #[strum(serialize = "collection")]
    Collection,
    #[serde(rename = "create-ls")]
    #[strum(serialize = "create-ls")]
    CreateLoadStatus,
    #[serde(rename = "create-ds")]
    #[strum(serialize = "create-ds")]
    CreateDataSource,
    #[serde(rename = "load")]
    #[strum(serialize = "load")]
    LoadData,
    #[serde(rename = "qa")]
    #[strum(serialize = "qa")]
    DataQualityAssurance,
}

impl RoleName {
    /// Gets the string representation of the [RoleName] as seen in the database
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Admin => "Role with full access to all other roles",
            Self::AddRole => {
                "Provides a user with the ability to add/remove roles from a user. However, this \
                 is limited to the roles of the current user."
            }
            Self::Check => "Provides a user the ability to check a data load instance",
            Self::Collection => {
                "Provides a user the ability to collect for and set up a data source"
            }
            Self::CreateLoadStatus => "Provides a user the ability to create a new load instance",
            Self::CreateDataSource => "Provides a user the ability to create a new data source",
            Self::LoadData => "Provides a user the ability to process data loads",
            Self::DataQualityAssurance => {
                "Provides a user the ability to perform quality assurance checks on a data load \
                 instance"
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

    /// Read all roles found in the database. Must be an admin user to access roles
    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<Role>>;
}
