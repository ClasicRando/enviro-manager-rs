use common::error::EmResult;
use sqlx::{PgPool, Pool, Postgres};

use crate::service::roles::{CreateRoleRequest, Role, RoleService, UpdateRoleRequest};

/// Postgresql implementation of [RoleService]
#[derive(Clone)]
pub struct PgRoleService {
    pool: PgPool,
}

impl RoleService for PgRoleService {
    type Database = Postgres;

    fn new(pool: &Pool<Self::Database>) -> Self {
        Self { pool: pool.clone() }
    }

    async fn read_many(&self) -> EmResult<Vec<Role>> {
        let result = sqlx::query_as(
            r#"
            select name, description
            from users.roles"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn create_role(&self, request: &CreateRoleRequest) -> EmResult<()> {
        sqlx::query("call users.create_role($1, $2, $3)")
            .bind(request.current_em_uid)
            .bind(&request.name)
            .bind(&request.description)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_role(&self, request: &UpdateRoleRequest) -> EmResult<()> {
        sqlx::query("call users.update_role($1, $2, $3)")
            .bind(request.current_em_uid)
            .bind(&request.name)
            .bind(&request.new_name)
            .bind(&request.new_description)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
