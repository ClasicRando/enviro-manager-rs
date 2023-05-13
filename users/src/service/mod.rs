use common::error::EmResult;
use sqlx::{Database, Pool};

use self::{roles::RoleService, users::UserService};

pub mod roles;
pub mod users;
pub mod postgres;

///
pub fn create_roles_service<R, D>(pool: &Pool<D>) -> EmResult<R>
where
    R: RoleService<Database = D>,
    D: Database,
{
    Ok(R::new(pool))
}

///
pub fn create_users_service<U, D>(pool: &Pool<D>) -> EmResult<U>
where
    U: UserService<Database = D>,
    D: Database,
{
    Ok(U::new(pool))
}
