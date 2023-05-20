use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool};
use uuid::Uuid;

use crate::service::users::UserService;

/// EnviroManager user role as a database entity
#[derive(Serialize, sqlx::FromRow)]
pub struct Role {
    /// Name of the role. Unique within all roles
    pub(crate) name: String,
    /// Short description of the role
    pub(crate) description: String,
}

/// All role names that exist as their common name
#[derive(Serialize)]
pub enum RoleName {
    Admin,
    CreateRole,
    AddRole,
}

impl RoleName {
    /// Gets the string representation of the [RoleName] as seen in the database
    pub const fn as_str(&self) -> &'static str {
        match self {
            RoleName::Admin => "admin",
            RoleName::CreateRole => "create-role",
            RoleName::AddRole => "add-role",
        }
    }
}

/// Request object to create a new role. Deserialized from an API request
#[derive(Deserialize)]
pub struct CreateRoleRequest {
    /// UID of the user attempting to create a new role
    pub(crate) current_uid: Uuid,
    /// Name of the new role
    pub(crate) name: String,
    /// Description of the new role
    pub(crate) description: String,
}

/// Request object to update an existing role. Deserialized from an API request
#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    /// UID of the user attempting to create a new role
    pub(crate) current_uid: Uuid,
    /// Name of the existing role
    pub(crate) name: String,
    /// New name for the role. If [None] role name will not change
    #[serde(default)]
    pub(crate) new_name: Option<String>,
    /// New description for the role. If [None] role description will not change
    #[serde(default)]
    pub(crate) new_description: Option<String>,
}

/// Service for interacting with the role system. Allows for reading all roles as well as creating
/// new and modifying existing roles. Requires the [UserService] as an associated type to fetch
/// user data to confirm the roles of a user before creating/modifying roles.
pub trait RoleService
where
    Self: Clone + Send + Sync,
{
    type Database: Database;
    type UserService: UserService<Database = Self::Database>;

    /// Create new instance of a [RoleService]. Both parameters are references to allow for cloning
    /// of the value.
    fn new(pool: &Pool<Self::Database>, user_service: &Self::UserService) -> Self;
    /// Read all roles found in the database
    async fn read_many(&self) -> EmResult<Vec<Role>>;
    /// Create a new role in the database. The user specified in `request` must have the
    /// 'create-role' role to perform this action.
    async fn create_role(&self, request: &CreateRoleRequest) -> EmResult<Role>;
    /// Update an existing role in the database. The user specified in `request` must have the
    /// 'create-role' role to perform this action.
    async fn update_role(&self, request: &UpdateRoleRequest) -> EmResult<Role>;
}
