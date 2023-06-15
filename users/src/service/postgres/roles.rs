use common::error::EmResult;
use sqlx::{
    database::HasArguments,
    decode::Decode,
    encode::IsNull,
    postgres::{PgHasArrayType, PgTypeInfo, PgValueRef},
    Encode, Postgres, Type,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::{
    data::role::{Role, RoleName},
    service::{postgres::users::PgUserService, roles::RoleService, users::UserService},
};

impl<'r> Decode<'r, Postgres> for Role {
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&'r str as Decode<'r, Postgres>>::decode(value)?;
        let name = match value {
            "admin" => RoleName::Admin,
            "add-role" => RoleName::AddRole,
            _ => return Err(format!("invalid value {value:?} for role name").into()),
        };
        let description = name.description();
        Ok(Self {
            name,
            description: description.to_owned(),
        })
    }
}

impl<'q> Encode<'q, Postgres> for Role
where
    RoleName: Encode<'q, Postgres>,
{
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <RoleName as Encode<'q, Postgres>>::encode(self.name, buf)
    }

    fn size_hint(&self) -> usize {
        <RoleName as Encode<'q, Postgres>>::size_hint(&self.name)
    }
}

impl<'q> Encode<'q, Postgres> for RoleName
where
    &'q str: Encode<'q, Postgres>,
{
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <&str as Encode<'q, Postgres>>::encode(self.into(), buf)
    }

    fn size_hint(&self) -> usize {
        <&str as Encode<'q, Postgres>>::size_hint(&self.into())
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

impl Type<Postgres> for RoleName {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl PgHasArrayType for RoleName {
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

impl PgRoleService {
    /// Create new instance of a [PgRoleService]. Both parameters are references to allow for
    /// cloning of the value.
    pub fn new(user_service: &PgUserService) -> Self {
        Self {
            user_service: user_service.clone(),
        }
    }
}

impl RoleService for PgRoleService {
    type UserService = PgUserService;

    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<Role>> {
        let user = self.user_service.read_one(current_uid).await?;
        user.check_role(RoleName::Admin)?;

        let roles = RoleName::iter()
            .map(|role_name| {
                let description = role_name.description();
                Role {
                    name: role_name,
                    description: description.to_owned(),
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
    use crate::{
        data::role::{Role, RoleName},
        service::{
            postgres::{test::database, users::PgUserService},
            roles::RoleService,
        },
    };

    #[rstest]
    #[case::privileged_user(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"))]
    #[tokio::test]
    async fn read_all_should_succeed_when(database: PgPool, #[case] uuid: Uuid) -> EmResult<()> {
        let service = PgRoleService::new(&PgUserService::new(&database));
        let static_roles: Vec<Role> = RoleName::iter()
            .map(|name| {
                let description = name.description();
                Role {
                    name,
                    description: description.to_owned(),
                }
            })
            .collect();

        let roles = service.read_all(&uuid).await?;

        assert_eq!(roles.len(), static_roles.len());
        assert_eq!(roles, static_roles);

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
