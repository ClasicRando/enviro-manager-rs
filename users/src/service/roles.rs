use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool};
use uuid::Uuid;

///
#[derive(Serialize, sqlx::FromRow)]
pub struct Role {
    name: String,
    description: String,
}

///
#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub(crate) current_em_uid: Uuid,
    pub(crate) name: String,
    pub(crate) description: String,
}

///
#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub(crate) current_em_uid: Uuid,
    pub(crate) name: String,
    pub(crate) new_name: String,
    pub(crate) new_description: String,
}

///
pub trait RoleService: Clone + Send + Sync {
    type Database: Database;

    ///
    fn new(pool: &Pool<Self::Database>) -> Self;
    ///
    async fn read_many(&self) -> EmResult<Vec<Role>>;
    ///
    async fn create_role(&self, request: &CreateRoleRequest) -> EmResult<()>;
    ///
    async fn update_role(&self, request: &UpdateRoleRequest) -> EmResult<()>;
}
