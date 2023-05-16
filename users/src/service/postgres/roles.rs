use common::error::EmResult;
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef,
    },
    PgPool, Pool, Postgres, Type,
};

use crate::service::roles::{CreateRoleRequest, Role, RoleService, UpdateRoleRequest};

impl Encode<'_, Postgres> for Role
where
    String: for<'q> Encode<'q, Postgres>,
    String: Type<Postgres>,
    String: for<'q> Encode<'q, Postgres>,
    String: Type<Postgres>,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);
        encoder.encode(&self.name);
        encoder.encode(&self.description);
        encoder.finish();
        IsNull::No
    }

    fn size_hint(&self) -> usize {
        2usize * (4 + 4)
            + <String as Encode<Postgres>>::size_hint(&self.name)
            + <String as Encode<Postgres>>::size_hint(&self.description)
    }
}

impl<'r> Decode<'r, Postgres> for Role
where
    String: Decode<'r, Postgres>,
    String: Type<Postgres>,
    String: Decode<'r, Postgres>,
    String: Type<Postgres>,
{
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let mut decoder = PgRecordDecoder::new(value)?;
        let name = decoder.try_decode::<String>()?;
        let description = decoder.try_decode::<String>()?;
        Ok(Role { name, description })
    }
}

impl Type<Postgres> for Role {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("roles")
    }
}

impl PgHasArrayType for Role {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_roles")
    }
}

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
        let role = sqlx::query_as("call users.update_role($1, $2, $3, $4, null, null)")
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
    use common::error::EmResult;
    use rstest::rstest;
    use sqlx::PgPool;
    use uuid::{uuid, Uuid};

    use super::PgRoleService;
    use crate::service::{
        postgres::test::database,
        roles::{CreateRoleRequest, RoleService},
    };

    fn create_role_request(uuid: Uuid, name: &str, description: &str) -> CreateRoleRequest {
        CreateRoleRequest {
            current_uid: uuid,
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn read_many_should_return_base_roles(database: &PgPool) -> EmResult<()> {
        let service = PgRoleService::new(database);

        let roles = service.read_many().await?;

        let admin_role = roles
            .iter()
            .find(|r| r.name == "admin")
            .expect("Could not find an `admin` role");

        assert_eq!(admin_role.description, "Role with full access to all other roles");

        let create_user_role = roles
            .iter()
            .find(|r| r.name == "create-user")
            .expect("Could not find an `create-user` role");

        assert_eq!(create_user_role.description, "Provides a user with the ability to create other users");

        let create_role_role = roles
            .iter()
            .find(|r| r.name == "create-role")
            .expect("Could not find an `create-role` role");

        assert_eq!(create_role_role.description, "Provides a user with the ability to create/modify roles");

        let add_role_role = roles
            .iter()
            .find(|r| r.name == "add-role")
            .expect("Could not find an `add-role` role");

        assert_eq!(add_role_role.description, "Provides a user with the ability to add/remove roles from a user");

        Ok(())
    }

    #[rstest]
    #[case(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "test", "This is a test role that should succeed")]
    #[tokio::test]
    async fn create_role_should_succeed_when_valid_request_as_admin(
        database: &PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(database);
        let request = create_role_request(uuid, name, description);
        let cleanup = async move {
            sqlx::query("delete from users.roles where name = $1")
                .bind(name)
                .execute(database)
                .await
        };

        let action = service.create_role(&request).await;
        cleanup.await?;

        let role = action?;

        assert_eq!(role.name, name);
        assert_eq!(role.description, description);

        Ok(())
    }

    #[rstest]
    #[case(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "test2", "This is a test role that should succeed")]
    #[tokio::test]
    async fn create_role_should_succeed_when_valid_request_without_admin(
        database: &PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(database);
        let request = create_role_request(uuid, name, description);
        let cleanup = async move {
            sqlx::query("delete from users.roles where name = $1")
                .bind(name)
                .execute(database)
                .await
        };

        let action = service.create_role(&request).await;
        cleanup.await?;

        let role = action?;

        assert_eq!(role.name, name);
        assert_eq!(role.description, description);

        Ok(())
    }

    #[rstest]
    #[case(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "admin", "This is a role that should not succeed")]
    #[tokio::test]
    async fn create_role_should_fail_when_role_name_already_exists(
        database: &PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(database);
        let request = create_role_request(uuid, name, description);

        let action = service.create_role(&request).await;

        assert!(action.is_err());
        Ok(())
    }

    #[rstest]
    #[case(uuid!("1cc58326-84aa-4c08-bb91-8c4536797e8c"), "missing-priv", "This is a role that should not succeed")]
    #[tokio::test]
    async fn create_role_should_fail_when_action_user_missing_privilege(
        database: &PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(database);
        let request = create_role_request(uuid, name, description);

        let action = service.create_role(&request).await;

        assert!(action.is_err());
        Ok(())
    }

    #[rstest]
    #[case(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "", "This is a role that should not succeed")]
    #[tokio::test]
    async fn create_role_should_fail_when_name_is_empty(
        database: &PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(database);
        let request = create_role_request(uuid, name, description);

        let action = service.create_role(&request).await;

        assert!(action.is_err());
        Ok(())
    }

    #[rstest]
    #[case(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "description-empty", "")]
    #[tokio::test]
    async fn create_role_should_fail_when_description_is_empty(
        database: &PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(database);
        let request = create_role_request(uuid, name, description);

        let action = service.create_role(&request).await;

        assert!(action.is_err());
        Ok(())
    }
}
