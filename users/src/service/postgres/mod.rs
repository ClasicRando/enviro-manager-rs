pub mod roles;
pub mod users;

#[cfg(test)]
mod test {
    use common::{
        database::{ConnectionBuilder, PgConnectionBuilder},
        error::{EmError, EmResult},
        execute_anonymous_block, package_dir, read_file,
    };
    use rstest::{fixture, rstest};
    use sqlx::PgPool;

    use crate::database::test_db_options;

    #[fixture]
    #[once]
    pub(crate) fn database() -> PgPool {
        let options = test_db_options().expect("Failed to create test database options");
        PgConnectionBuilder::create_pool_lazy(options, 20, 0)
    }

    #[rstest]
    #[case("users/check_user_role.pgsql")]
    #[case("users/validate_password.pgsql")]
    #[tokio::test]
    async fn database_test(database: &PgPool, #[case] test_path: &str) -> EmResult<()> {
        let path = package_dir().join("database/tests").join(test_path);
        let Ok(block) = read_file(&path).await else {
            return Err(EmError::Generic(format!("Could not open file '{:?}'", path)))
        };
        execute_anonymous_block(&block, database).await?;
        Ok(())
    }
}
