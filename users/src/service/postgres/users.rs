use common::{
    api::ApiRequestValidator,
    database::{
        connection::{finalize_transaction, get_connection_with_em_uid},
        postgres::Postgres,
    },
    error::{
        EmError::{self, InvalidUser},
        EmResult,
    },
};
use sqlx::{Connection, PgPool};
use uuid::Uuid;

use crate::{
    data::{role::RoleName, user::User},
    service::users::{
        CreateUserRequest, CreateUserRequestValidator, ModifyUserRoleRequest, UpdateUserRequest,
        UpdateUserRequestValidator, UserService, ValidateUserRequest,
    },
};

/// Postgresql implementation of [UserService]
#[derive(Clone)]
pub struct PgUserService {
    /// Postgres database connection pool used by this service
    pool: PgPool,
}

impl PgUserService {
    /// Create new instance of a [UserService]
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    /// Update the password of a user with the `uid` specified
    #[allow(unused)]
    async fn reset_password(&self, uid: &Uuid, new_password: &str) -> EmResult<()> {
        let mut connection = get_connection_with_em_uid(uid, &self.pool).await?;
        sqlx::query("call users.reset_password($1, $2)")
            .bind(uid)
            .bind(new_password)
            .execute(&mut connection)
            .await?;
        Ok(())
    }
}

impl UserService for PgUserService {
    type CreateRequestValidator = CreateUserRequestValidator;
    type Database = Postgres;
    type UpdateRequestValidator = UpdateUserRequestValidator;

    async fn create_user(&self, current_uid: &Uuid, request: &CreateUserRequest) -> EmResult<User> {
        Self::CreateRequestValidator::validate(request)?;
        let CreateUserRequest {
            full_name,
            username,
            password,
            roles,
        } = request;
        let user = self.read_one(current_uid).await?;
        user.check_role(RoleName::Admin)?;

        let mut connection = get_connection_with_em_uid(current_uid, &self.pool).await?;
        let mut transaction = connection.begin().await?;
        let uid: Uuid = sqlx::query_scalar("select users.create_user($1, $2, $3)")
            .bind(full_name)
            .bind(username)
            .bind(password)
            .fetch_one(&mut transaction)
            .await?;

        let role_add_result = sqlx::query("call users.add_user_roles($1, $2)")
            .bind(uid)
            .bind(roles)
            .execute(&mut transaction)
            .await;
        finalize_transaction(role_add_result, transaction).await?;
        // Drop connection to clean up the resource after the transaction is finished
        drop(connection);

        self.read_one(&uid).await
    }

    async fn read_all(&self, current_uid: &Uuid) -> EmResult<Vec<User>> {
        let user = self.read_one(current_uid).await?;
        user.check_role(RoleName::Admin)?;

        let users = sqlx::query_as(
            r#"
            select u.uid, u.username, u.full_name, u.roles
            from users.v_users u"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    async fn read_one(&self, uuid: &Uuid) -> EmResult<User> {
        let user = sqlx::query_as(
            r#"
            select u.uid, u.username, u.full_name, u.roles
            from users.v_users u
            where u.uid = $1"#,
        )
        .bind(uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn update(&self, current_uid: &Uuid, request: &UpdateUserRequest) -> EmResult<User> {
        let user = self.read_one(current_uid).await?;
        if user.check_role(RoleName::Admin).is_err() {
            return Err(EmError::InvalidUser);
        }

        Self::UpdateRequestValidator::validate(request)?;
        let UpdateUserRequest {
            update_uid,
            new_name,
            new_username,
        } = request;

        let mut connection = get_connection_with_em_uid(current_uid, &self.pool).await?;
        sqlx::query("call users.update_user($1, $2, $3)")
            .bind(update_uid)
            .bind(new_name)
            .bind(new_username)
            .execute(&mut connection)
            .await?;
        self.read_one(update_uid).await
    }

    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User> {
        let ValidateUserRequest { username, password } = request;
        let result = sqlx::query_as(
            r#"
            select v.uid, v.username, v.full_name, v.roles
            from users.validate_user($1, $2) v"#,
        )
        .bind(username)
        .bind(password)
        .fetch_optional(&self.pool)
        .await?;
        result.map_or_else(|| Err(InvalidUser), Ok)
    }

    async fn modify_user_role(
        &self,
        current_uid: &Uuid,
        request: &ModifyUserRoleRequest,
    ) -> EmResult<User> {
        let ModifyUserRoleRequest { uid, role, add } = request;

        let user = self.read_one(current_uid).await?;
        user.check_role(RoleName::AddRole)?;
        user.check_role(*role)?;

        let query = if *add {
            "call users.add_user_role($1, $2)"
        } else {
            "call users.revoke_user_role($1, $2)"
        };
        sqlx::query(query)
            .bind(uid)
            .bind(role)
            .execute(&self.pool)
            .await?;
        self.read_one(uid).await
    }
}

#[cfg(test)]
mod test {

    use common::error::EmResult;
    use rstest::rstest;
    use sqlx::PgPool;
    use uuid::{uuid, Uuid};

    use super::PgUserService;
    use crate::{
        data::role::RoleName,
        service::{
            postgres::test::database,
            users::{
                test::{create_user_request, validate_user_request},
                CreateUserRequest, UserService, ValidateUserRequest,
            },
        },
    };

    /// Cleanup function for users that are created during tests
    async fn cleanup_user_create(username: &str, pool: &PgPool) -> EmResult<()> {
        sqlx::query("delete from users.users where username = $1")
            .bind(username)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[rstest]
    #[case::valid_request(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), create_user_request("Mr Test", "test", "Test1!", &["admin"]))]
    #[tokio::test]
    async fn create_user_should_succeed_when(
        database: PgPool,
        #[case] uid: Uuid,
        #[case] user_request: CreateUserRequest,
    ) -> EmResult<()> {
        let service = PgUserService::new(&database);

        let action = service.create_user(&uid, &user_request).await;
        cleanup_user_create(&user_request.username, &database).await?;

        let user = action?;
        let user_roles: Vec<RoleName> = user.roles.iter().map(|r| r.name).collect();

        assert_eq!(user.full_name, user_request.full_name);
        assert_eq!(user_roles, user_request.roles);

        Ok(())
    }

    #[rstest]
    #[case::user_does_not_exist(Uuid::new_v4(), create_user_request("Mr Test2", "test2", "Test1!", &["admin"]))]
    #[case::missing_privilege(uuid!("728ac060-9d38-47e9-b2fa-66d2954110e3"), create_user_request("Mr Test3", "test3", "Test1!", &["admin"]))]
    #[case::username_exists(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), create_user_request("Mr Test4", "none", "Test1!", &["admin"]))]
    #[tokio::test]
    async fn create_user_should_fail_when(
        database: PgPool,
        #[case] uid: Uuid,
        #[case] user_request: CreateUserRequest,
    ) -> EmResult<()> {
        let service = PgUserService::new(&database);

        let action = service.create_user(&uid, &user_request).await;
        if user_request.username != "none" {
            cleanup_user_create(&user_request.username, &database).await?;
        }

        assert!(action.is_err());

        Ok(())
    }

    #[rstest]
    #[case::admin(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "Admin This is", vec!["admin"])]
    #[case::add_role(uuid!("728ac060-9d38-47e9-b2fa-66d2954110e3"), "Add-Role This is", vec!["add-role"])]
    #[case::add_role(uuid!("be4c1ef7-771a-4580-b0dd-ff137c64ab48"), "None This is", vec![])]
    #[tokio::test]
    async fn read_all_should_contain(
        database: PgPool,
        #[case] uuid: Uuid,
        #[case] full_name: &str,
        #[case] roles: Vec<&str>,
    ) -> EmResult<()> {
        let service = PgUserService::new(&database);

        let users = service
            .read_all(&uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"))
            .await?;

        let user = users
            .iter()
            .find(|u| u.uid == uuid)
            .ok_or("Could not find admin user")?;
        let user_roles: Vec<&str> = user.roles.iter().map(|r| r.name.as_ref()).collect();
        assert_eq!(user.full_name, full_name);
        assert_eq!(user_roles, roles);

        Ok(())
    }

    #[rstest]
    #[case::valid_request(validate_user_request("admin", "admin"), uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"))]
    #[tokio::test]
    async fn validate_user_should_succeed_when(
        database: PgPool,
        #[case] validate_user_request: ValidateUserRequest,
        #[case] uuid: Uuid,
    ) -> EmResult<()> {
        let service = PgUserService::new(&database);

        let user = service.validate_user(&validate_user_request).await?;

        assert_eq!(user.uid, uuid);

        Ok(())
    }

    #[rstest]
    #[case::invalid_username(validate_user_request("test", "admin"))]
    #[case::invalid_password(validate_user_request("admin", "test"))]
    #[tokio::test]
    async fn validate_user_should_fail_when(
        database: PgPool,
        #[case] validate_user_request: ValidateUserRequest,
    ) -> EmResult<()> {
        let service = PgUserService::new(&database);

        let action = service.validate_user(&validate_user_request).await;

        assert!(action.is_err());

        Ok(())
    }
}
