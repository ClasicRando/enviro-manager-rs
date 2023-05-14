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
            from users.v_roles"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn create_role(&self, request: &CreateRoleRequest) -> EmResult<Role> {
        let role = sqlx::query_as("call users.create_role($1, $2, $3, null, null)")
            .bind(request.current_uid)
            .bind(&request.name)
            .bind(&request.description)
            .fetch_one(&self.pool)
            .await?;
        Ok(role)
    }

    async fn update_role(&self, request: &UpdateRoleRequest) -> EmResult<Role> {
        let role = sqlx::query_as("call users.update_role($1, $2, $3)")
            .bind(request.current_uid)
            .bind(&request.name)
            .bind(&request.new_name)
            .bind(&request.new_description)
            .fetch_one(&self.pool)
            .await?;
        Ok(role)
    }
}

#[cfg(test)]
mod test {
    use common::{
        database::{ConnectionBuilder, PgConnectionBuilder},
        error::EmResult,
    };
    use rstest::{fixture, rstest};
    use sqlx::PgPool;
    use uuid::{uuid, Uuid};

    use super::PgRoleService;
    use crate::{
        database::test_db_options,
        service::roles::{CreateRoleRequest, RoleService},
    };

    #[fixture]
    async fn database_pool() -> EmResult<PgPool> {
        PgConnectionBuilder::create_pool(test_db_options()?, 1, 1).await
    }

    fn create_role_request(uuid: Uuid, name: &str, description: &str) -> CreateRoleRequest {
        CreateRoleRequest {
            current_uid: uuid,
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    #[rstest]
    #[case(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "test", "This is a test role that should succeed")]
    #[tokio::test]
    async fn create_role_should_succeed_when_valid_request(
        #[future] database_pool: EmResult<PgPool>,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let pool = database_pool.await?;
        let service = PgRoleService::new(&pool);
        let request = create_role_request(uuid, name, description);
        let cleanup = async move {
            sqlx::query("delete from users.roles where name = $1")
                .bind(name)
                .execute(&pool)
                .await
        };

        let action = service.create_role(&request).await;
        cleanup.await?;

        let role = action?;

        assert_eq!(role.name, name);
        assert_eq!(role.description, description);

        Ok(())
    }
}
