use common::error::{EmError, EmResult};
use sqlx::{PgPool, Pool, Postgres};

use crate::service::users::{
    CreateUserRequest, ModifyUserRoleRequest, UpdateUserRequest, UpdateUserType, User, UserService,
    ValidateUserRequest,
};

/// Postgresql implementation of [UserService]
#[derive(Clone)]
pub struct PgUserService {
    pool: PgPool,
}

impl PgUserService {
    async fn update_full_name(
        &self,
        username: &str,
        password: &str,
        new_first_name: &str,
        new_last_name: &str,
    ) -> EmResult<User> {
        let user = sqlx::query_as("call users.update_full_name($1, $2, $3, $4)")
            .bind(username)
            .bind(password)
            .bind(new_first_name)
            .bind(new_last_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    async fn update_username(
        &self,
        username: &str,
        password: &str,
        new_username: &str,
    ) -> EmResult<User> {
        let user = sqlx::query_as("call users.update_username($1, $2, $3)")
            .bind(username)
            .bind(password)
            .bind(new_username)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    async fn reset_password(
        &self,
        username: &str,
        password: &str,
        new_password: &str,
    ) -> EmResult<User> {
        let user = sqlx::query_as("call users.reset_password($1, $2, $3)")
            .bind(username)
            .bind(password)
            .bind(new_password)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
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
        let result = sqlx::query_as("select users.create_user($1, $2, $3, $4, $5, $6)")
            .bind(current_uid)
            .bind(first_name)
            .bind(last_name)
            .bind(username)
            .bind(password)
            .bind(roles)
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }

    async fn update(&self, request: &UpdateUserRequest) -> EmResult<User> {
        let UpdateUserRequest {
            username,
            password,
            update_type,
        } = request;
        match update_type {
            UpdateUserType::Username { new_username } => {
                self.update_username(username, password, new_username).await
            }
            UpdateUserType::FullName {
                new_first_name,
                new_last_name,
            } => {
                self.update_full_name(username, password, new_first_name, new_last_name)
                    .await
            }
            UpdateUserType::ResetPassword { new_password } => {
                self.reset_password(username, password, new_password).await
            }
        }
    }

    async fn validate_user(&self, request: &ValidateUserRequest) -> EmResult<User> {
        let ValidateUserRequest { username, password } = request;
        let result = sqlx::query_as("select users.validate_user($1, $2)")
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
        let query = if *add {
            "call users.add_user_role($1, $2, $3)"
        } else {
            "call users.revoke_user_role($1, $2, $3)"
        };
        let user = sqlx::query_as(query)
            .bind(current_uid)
            .bind(uid)
            .bind(role)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
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

    use super::PgUserService;
    use crate::{
        database::test_db_options,
        service::users::{CreateUserRequest, UserService},
    };

    #[fixture]
    async fn database_pool() -> EmResult<PgPool> {
        PgConnectionBuilder::create_pool(test_db_options()?, 1, 1).await
    }

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

    #[rstest]
    #[case(uuid!("9363ab3f-0d62-4b40-b408-898bdea56282"), "Mr", "Test", "test", "Test1!", vec!["admin"])]
    #[tokio::test]
    async fn create_user_should_succeed_when_valid_request(
        #[future] database_pool: EmResult<PgPool>,
        #[case] uuid: Uuid,
        #[case] first_name: &str,
        #[case] last_name: &str,
        #[case] username: &str,
        #[case] password: &str,
        #[case] roles: Vec<&str>,
    ) -> EmResult<()> {
        let pool = database_pool.await?;
        let service = PgUserService::new(&pool);
        let user_request =
            create_user_request(uuid, first_name, last_name, username, password, &roles);
        let cleanup = async move {
            sqlx::query("delete from users.users where username = $1")
                .bind(username)
                .execute(&pool)
                .await
        };

        let action = service.create_user(&user_request).await;
        cleanup.await?;

        let user = action?;
        let user_roles: Vec<&str> = user.roles.iter().map(|r| r.name.as_str()).collect();

        assert_eq!(user.full_name, format!("{} {}", first_name, last_name));
        assert_eq!(user_roles, roles);

        Ok(())
    }
}
