use common::error::EmResult;
use serde::{Deserialize, Serialize};
use sqlx::{Database, PgPool, Pool, Postgres};

use crate::services::users::EmUid;

///
#[derive(Serialize, sqlx::FromRow)]
pub struct Role {
    name: String,
    description: String,
}

///
#[derive(Deserialize)]
pub struct CreateRoleRequest {
    current_em_uid: EmUid,
    name: String,
    description: String,
}

impl CreateRoleRequest {
    ///
    pub fn name(&self) -> &str {
        &self.name
    }
}

///
#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    current_em_uid: EmUid,
    name: String,
    new_name: String,
    new_description: String,
}

impl UpdateRoleRequest {
    ///
    pub fn name(&self) -> &str {
        &self.name
    }
}

///
#[async_trait::async_trait]
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

/// Postgresql implementation of [RoleService]
#[derive(Clone)]
pub struct PgRoleService {
    pool: PgPool,
}

#[async_trait::async_trait]
impl RoleService for PgRoleService {
    type Database = Postgres;

    fn new(pool: &Pool<Self::Database>) -> Self {
        Self { pool: pool.clone() }
    }

    async fn read_many(&self) -> EmResult<Vec<Role>> {
        let result = sqlx::query_as(
            r#"
            select name, description
            from enviro_manager_user.roles"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn create_role(&self, request: &CreateRoleRequest) -> EmResult<()> {
        sqlx::query("call enviro_manager_user.create_role($1, $2, $3)")
            .bind(&request.current_em_uid)
            .bind(&request.name)
            .bind(&request.description)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_role(&self, request: &UpdateRoleRequest) -> EmResult<()> {
        sqlx::query("call enviro_manager_user.update_role($1, $2, $3)")
            .bind(&request.current_em_uid)
            .bind(&request.name)
            .bind(&request.new_name)
            .bind(&request.new_description)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
