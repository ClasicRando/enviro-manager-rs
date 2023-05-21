use common::{
    database::connection::{finalize_transaction, get_connection_with_em_uid},
    error::{EmError, EmResult},
};
use lazy_regex::regex;
use sqlx::{Connection, PgPool, Pool, Postgres};
use uuid::Uuid;

use crate::service::{
    roles::RoleName,
    users::{
        CreateUserRequest, ModifyUserRoleRequest, UpdateUserRequest, UpdateUserType, User,
        UserService, ValidateUserRequest,
    },
};

/// Postgresql implementation of [UserService]
#[derive(Clone)]
pub struct PgUserService {
    /// Postgres database connection pool used by this service
    pool: PgPool,
}

impl PgUserService {
    /// Update the full name of a user with the `uid` specified
    async fn update_full_name(
        &self,
        uid: &Uuid,
        new_first_name: &str,
        new_last_name: &str,
    ) -> EmResult<()> {
        let mut connection = get_connection_with_em_uid(uid, &self.pool).await?;
        sqlx::query("call users.update_full_name($1, $2, $3)")
            .bind(uid)
            .bind(new_first_name)
            .bind(new_last_name)
            .execute(&mut connection)
            .await?;
        Ok(())
    }

    /// Update the username of a user with the `uid` specified
    async fn update_username(&self, uid: &Uuid, new_username: &str) -> EmResult<()> {
        let mut connection = get_connection_with_em_uid(uid, &self.pool).await?;
        sqlx::query("call users.update_username($1, $2)")
            .bind(uid)
            .bind(new_username)
            .execute(&mut connection)
            .await?;
        Ok(())
    }

    /// Update the password of a user with the `uid` specified
    async fn reset_password(&self, uid: &Uuid, new_password: &str) -> EmResult<()> {
        let mut connection = get_connection_with_em_uid(uid, &self.pool).await?;
        sqlx::query("call users.reset_password($1, $2)")
            .bind(uid)
            .bind(new_password)
            .execute(&mut connection)
            .await?;
        Ok(())
    }

    /// Validate that the provided `password` meets the rules prescribed for password
    fn validate_password(&self, password: &str) -> EmResult<()> {
        if password.is_empty() {
            return Err(EmError::InvalidPassword {
                reason: "Must not be null or an empty string",
            });
        }
        if !regex!("[A-Z]").is_match(password) {
            return Err(EmError::InvalidPassword {
                reason: "Must contain at least 1 uppercase character",
            });
        }
        if !regex!(r"\d").is_match(password) {
            return Err(EmError::InvalidPassword {
                reason: "Must contain at least 1 digit character",
            });
        }
        if !regex!(r"\W").is_match(password) {
            return Err(EmError::InvalidPassword {
                reason: "Must contain at least 1 non-alphanumeric character",
            });
        }
        Ok(())
    }
}

impl UserService for PgUserService {
    type Database = Postgres;

    fn new(pool: &Pool<Self::Database>) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create_user(&self, request: &CreateUserRequest) -> EmResult<User> {
        let CreateUserRequest {
            current_uid,
            first_name,
            last_name,
            username,
            password,
            roles,
        } = request;
        let user = self.read_one(current_uid).await?;

        user.check_role(RoleName::Admin)?;
        self.validate_password(password)?;

        let mut connection = get_connection_with_em_uid(current_uid, &self.pool).await?;
        let mut transaction = connection.begin().await?;
        let uid: Uuid = sqlx::query_scalar("select users.create_user($1, $2, $3, $4)")
            .bind(first_name)
            .bind(last_name)
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
            select u.uid, u.full_name, u.roles
            from users.v_users u"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    async fn read_one(&self, uuid: &Uuid) -> EmResult<User> {
        let user = sqlx::query_as(
            r#"
            select u.uid, u.full_name, u.roles
            from users.v_users u
            where u.uid = $1"#,
        )
        .bind(uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn update(&self, request: &UpdateUserRequest) -> EmResult<User> {
        let UpdateUserRequest {
            validate_user,
            update_type,
        } = request;
        let user = self.validate_user(validate_user).await?;
        match update_type {
            UpdateUserType::Username { new_username } => {
                self.update_username(&user.uid, new_username).await?
            }
            UpdateUserType::FullName {
                new_first_name,
                new_last_name,
            } => {
                self.update_full_name(&user.uid, new_first_name, new_last_name)
                    .await?
            }
            UpdateUserType::ResetPassword { new_password } => {
                self.validate_password(new_password)?;
                self.reset_password(&user.uid, new_password).await?
            }
        }
        self.read_one(&user.uid).await
    }

    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User> {
        let ValidateUserRequest { username, password } = request;
        let result = sqlx::query_as(
            r#"
            select v.uid, v.full_name, v.role
            from users.validate_user($1, $2) v"#,
        )
        .bind(username)
        .bind(password)
        .fetch_optional(&self.pool)
        .await?;
        match result {
            Some(user) => Ok(user),
            None => Err(EmError::InvalidUser),
        }
    }

    async fn modify_user_role(&self, request: &ModifyUserRoleRequest) -> EmResult<User> {
        let ModifyUserRoleRequest {
            current_uid,
            uid,
            role,
            add,
        } = request;

        let user = self.read_one(current_uid).await?;
        user.check_role(RoleName::AddRole)?;

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
    use crate::service::{
        postgres::test::database,
        users::{CreateUserRequest, UserService},
    };

    /// Utility method for creating a new [CreateUserRequest]
    fn create_user_request(
        uuid: Uuid,
        first_name: &str,
        last_name: &str,
        username: &str,
        password: &str,
        roles: &[&str],
    ) -> CreateUserRequest {
        CreateUserRequest {
            current_uid: uuid,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
        }
    }

    /// Cleanup function for users that are created during tests
    async fn cleanup_user_create(username: &str, pool: &PgPool) -> EmResult<()> {
        sqlx::query("delete from users.users where username = $1")
            .bind(username)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[rstest]
    #[case(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "Mr", "Test", "test", "Test1!", vec!["admin"])]
    #[tokio::test]
    async fn create_user_should_succeed_when_valid_request(
        database: PgPool,
        #[case] uuid: Uuid,
        #[case] first_name: &str,
        #[case] last_name: &str,
        #[case] username: &str,
        #[case] password: &str,
        #[case] roles: Vec<&str>,
    ) -> EmResult<()> {
        let service = PgUserService::new(&database);
        let user_request =
            create_user_request(uuid, first_name, last_name, username, password, &roles);

        let action = service.create_user(&user_request).await;
        cleanup_user_create(username, &database).await?;

        let user = action?;
        let user_roles: Vec<&str> = user.roles.iter().map(|r| r.name.as_str()).collect();

        assert_eq!(user.full_name, format!("{} {}", first_name, last_name));
        assert_eq!(user_roles, roles);

        Ok(())
    }
}
