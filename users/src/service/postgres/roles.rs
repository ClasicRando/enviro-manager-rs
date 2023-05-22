use common::error::EmResult;
use sqlx::{
    decode::Decode,
    postgres::{PgHasArrayType, PgTypeInfo, PgValueRef},
    Postgres, Type,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::service::{
    postgres::users::PgUserService,
    roles::{Role, RoleName, RoleService},
    users::UserService,
};

impl<'r> Decode<'r, Postgres> for Role {
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&'r str as Decode<'r, Postgres>>::decode(value)?;
        let name = match value {
            "admin" => RoleName::Admin,
            "add-role" => RoleName::AddRole,
            _ => return Err(format!("invalid value {:?} for role name", value).into()),
        };
        let description = name.description();
        Ok(Role { name, description })
    }
}

impl Type<Postgres> for Role {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl PgHasArrayType for Role {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_text")
    }
}

/// Postgresql implementation of [RoleService]
#[derive(Clone)]
pub struct PgRoleService {
    /// Postgres [UserService] to allow for this service to fetch user data
    user_service: PgUserService,
}

impl RoleService for PgRoleService {
    type UserService = PgUserService;

    fn new(user_service: &Self::UserService) -> Self {
        Self {
            user_service: user_service.clone(),
        }
    }

    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<Role>> {
        let user = self.user_service.read_one(current_uid).await?;
        user.check_role(RoleName::Admin)?;

        let roles = RoleName::iter()
            .map(|role_name| {
                let description = role_name.description();
                Role {
                    name: role_name,
                    description,
                }
            })
            .collect();
        Ok(roles)
    }
}

#[cfg(test)]
mod test {
    use common::error::EmResult;
    use rstest::rstest;
    use sqlx::PgPool;
    use strum::IntoEnumIterator;
    use uuid::{uuid, Uuid};

    use super::PgRoleService;
    use crate::service::{
        postgres::{test::database, users::PgUserService},
        roles::{RoleName, RoleService},
        users::UserService,
    };

    #[rstest]
    #[case::privileged_user(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"))]
    #[tokio::test]
    async fn read_all_should_succeed_when(database: PgPool, #[case] uuid: Uuid) -> EmResult<()> {
        let service = PgRoleService::new(&PgUserService::new(&database));
        let role_count = RoleName::iter().count();

        let roles = service.read_all(&uuid).await?;

        assert_eq!(roles.len(), role_count);

        Ok(())
    }

    #[rstest]
    #[case::non_privileged_user(uuid!("be4c1ef7-771a-4580-b0dd-ff137c64ab48"))]
    #[tokio::test]
    async fn read_all_should_fail_when(database: PgPool, #[case] uuid: Uuid) -> EmResult<()> {
        let service = PgRoleService::new(&PgUserService::new(&database));

        let result = service.read_all(&uuid).await;

        assert!(result.is_err());

        Ok(())
    }
}
