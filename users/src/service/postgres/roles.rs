use common::{
    database::get_connection_with_em_uid,
    error::{EmError, EmResult},
};
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef,
    },
    PgPool, Pool, Postgres, Type,
};

use crate::service::{
    postgres::users::PgUserService,
    roles::{CreateRoleRequest, Role, RoleName, RoleService, UpdateRoleRequest},
    users::UserService,
};

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
    /// Postgres database connection pool used by this service
    pool: PgPool,
    /// Postgres [UserService] to allow for this service to fetch user data
    user_service: PgUserService,
}

impl RoleService for PgRoleService {
    type Database = Postgres;
    type UserService = PgUserService;

    fn new(pool: &Pool<Self::Database>, user_service: &Self::UserService) -> Self {
        Self {
            pool: pool.clone(),
            user_service: user_service.clone(),
        }
    }

    async fn read_all(&self) -> EmResult<Vec<Role>> {
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
        let CreateRoleRequest {
            current_uid,
            name,
            description,
        } = request;
        let user = self.user_service.read_one(current_uid).await?;
        user.check_role(RoleName::CreateRole)?;

        let mut connection = get_connection_with_em_uid(current_uid, &self.pool).await?;
        let role_option = sqlx::query_as(
            r#"
            select r.name, r.description
            from users.create_role($1, $2) r"#,
        )
        .bind(name)
        .bind(description)
        .fetch_optional(&mut connection)
        .await?;
        match role_option {
            Some(role) => Ok(role),
            None => Err(EmError::MissingRecord {
                pk: format!("{}", request.name),
            }),
        }
    }

    async fn update_role(&self, request: &UpdateRoleRequest) -> EmResult<Role> {
        let UpdateRoleRequest {
            current_uid,
            name,
            new_name,
            new_description,
        } = request;
        let user = self.user_service.read_one(current_uid).await?;
        user.check_role(RoleName::CreateRole)?;

        let mut connection = get_connection_with_em_uid(current_uid, &self.pool).await?;
        user.check_role(RoleName::CreateRole)?;
        let role_option = sqlx::query_as(
            r#"
            select r.name, r.description
            from users.update_role($1, $2, $3) r"#,
        )
        .bind(name)
        .bind(new_name)
        .bind(new_description)
        .fetch_optional(&mut connection)
        .await?;
        match role_option {
            Some(role) => Ok(role),
            None => Err(EmError::MissingRecord {
                pk: format!("{}", name),
            }),
        }
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
        postgres::{test::database, users::PgUserService},
        roles::{CreateRoleRequest, RoleService, UpdateRoleRequest},
        users::UserService,
    };

    fn create_role_request(uuid: Uuid, name: &str, description: &str) -> CreateRoleRequest {
        CreateRoleRequest {
            current_uid: uuid,
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    fn update_role_request(
        uuid: Uuid,
        name: &str,
        new_name: &str,
        new_description: &str,
    ) -> UpdateRoleRequest {
        UpdateRoleRequest {
            current_uid: uuid,
            name: name.to_string(),
            new_name: if new_name.trim().is_empty() {
                None
            } else {
                Some(new_name.to_string())
            },
            new_description: if new_description.trim().is_empty() {
                None
            } else {
                Some(new_description.to_string())
            },
        }
    }

    #[rstest]
    #[tokio::test]
    async fn read_many_should_return_base_roles(database: PgPool) -> EmResult<()> {
        let service = PgRoleService::new(&database, &PgUserService::new(&database));

        let roles = service.read_all().await?;

        let admin_role = roles
            .iter()
            .find(|r| r.name == "admin")
            .expect("Could not find an `admin` role");

        assert_eq!(
            admin_role.description,
            "Role with full access to all other roles"
        );

        let create_role_role = roles
            .iter()
            .find(|r| r.name == "create-role")
            .expect("Could not find an `create-role` role");

        assert_eq!(
            create_role_role.description,
            "Provides a user with the ability to create/modify roles"
        );

        let add_role_role = roles
            .iter()
            .find(|r| r.name == "add-role")
            .expect("Could not find an `add-role` role");

        assert_eq!(
            add_role_role.description,
            "Provides a user with the ability to add/remove roles from a user"
        );

        Ok(())
    }

    #[rstest]
    #[case::valid_request(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "test", "This is a test role that should succeed")]
    #[tokio::test]
    async fn create_role_should_succeed_when(
        database: PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(&database, &PgUserService::new(&database));
        let request = create_role_request(uuid, name, description);
        let cleanup = async move {
            sqlx::query("delete from users.roles where name = $1")
                .bind(name)
                .execute(&database)
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
    #[case::role_name_already_exists(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "admin", "This is a role that should not succeed")]
    #[case::action_user_missing_privilege(uuid!("1cc58326-84aa-4c08-bb91-8c4536797e8c"), "missing-priv", "This is a role that should not succeed")]
    #[case::name_is_empty(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "", "This is a role that should not succeed")]
    #[case::description_is_empty(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "description-empty", "")]
    #[tokio::test]
    async fn create_role_should_fail_when(
        database: PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(&database, &PgUserService::new(&database));
        let request = create_role_request(uuid, name, description);
        let cleanup = async move {
            sqlx::query("delete from users.roles where name = $1 and description = $2")
                .bind(name)
                .bind(description)
                .execute(&database)
                .await
        };

        let action = service.create_role(&request).await;
        cleanup.await?;

        assert!(action.is_err());
        Ok(())
    }

    async fn cleanup_role_update(
        name: &str,
        result_name: &str,
        pool: &PgPool,
    ) -> EmResult<()> {
        sqlx::query(
            r#"
            update users.roles
            set
                name = $1,
                description = 'Test role to update'
            where name = $2"#,
        )
        .bind(name)
        .bind(result_name)
        .execute(pool)
        .await?;
        Ok(())
    }

    #[rstest]
    #[case::valid_request_to_update_both_fields(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "update-role-1", "update-role-1-1", "New Description")]
    #[case::valid_request_to_update_name_only(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "update-role-2", "update-role-2-1", "")]
    #[case::valid_request_to_update_description(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "update-role-3", "", "New Description")]
    #[case::request_to_update_nothing(uuid!("bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca"), "update-role-4", "", "")]
    #[tokio::test]
    async fn update_role_should_succeed_when(
        database: PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] new_name: &str,
        #[case] new_description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(&database, &PgUserService::new(&database));
        let request = update_role_request(uuid, name, new_name, new_description);
        let result_name = if new_name.is_empty() { name } else { new_name };
        let result_description = if new_description.is_empty() {
            "Test role to update"
        } else {
            new_description
        };

        let action = service.update_role(&request).await;
        cleanup_role_update(name, result_name, &database).await?;
        let role = action?;

        assert_eq!(role.name, result_name);
        assert_eq!(role.description, result_description);

        Ok(())
    }

    #[rstest]
    #[case::action_user_missing_privilege(uuid!("1cc58326-84aa-4c08-bb91-8c4536797e8c"), "update-role-5", "", "")]
    #[tokio::test]
    async fn update_role_should_fail_when(
        database: PgPool,
        #[case] uuid: Uuid,
        #[case] name: &str,
        #[case] new_name: &str,
        #[case] new_description: &str,
    ) -> EmResult<()> {
        let service = PgRoleService::new(&database, &PgUserService::new(&database));
        let request = update_role_request(uuid, name, new_name, new_description);
        let result_name = if new_name.is_empty() { name } else { new_name };

        let action = service.update_role(&request).await;
        cleanup_role_update(name, result_name, &database).await?;

        assert!(action.is_err());
        Ok(())
    }
}
