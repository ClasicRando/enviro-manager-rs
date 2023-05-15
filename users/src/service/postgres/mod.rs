pub mod roles;
pub mod users;

#[cfg(test)]
mod test {
    use common::database::{ConnectionBuilder, PgConnectionBuilder};
    use rstest::fixture;
    use sqlx::PgPool;

    use crate::database::test_db_options;

    #[fixture]
    #[once]
    pub(crate) fn database() -> PgPool {
        let options = test_db_options().expect("Failed to create test database options");
        PgConnectionBuilder::create_pool_lazy(options, 20, 0)
    }
}
