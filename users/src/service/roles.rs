use common::error::EmResult;
use uuid::Uuid;

use crate::{data::role::Role, service::users::UserService};

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
