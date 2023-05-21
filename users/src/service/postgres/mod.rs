pub mod roles;
pub mod users;

#[cfg(test)]
mod test {
    use common::database::connection::{ConnectionBuilder, PgConnectionBuilder};
    use rstest::fixture;
    use sqlx::PgPool;

    use crate::database::test_db_options;

    #[fixture]
    pub(crate) fn database() -> PgPool {
        let options = test_db_options().expect("Failed to create test database options");
        PgConnectionBuilder::create_pool_lazy(options, 1, 1)
    }
}
