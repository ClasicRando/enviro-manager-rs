pub mod roles;
pub mod users;

#[cfg(test)]
mod test {
    use common::database::{
        connection::ConnectionBuilder, postgres::connection::PgConnectionBuilder,
    };
    use rstest::fixture;
    use sqlx::PgPool;

    use crate::database::db_options;

    #[fixture]
    pub(crate) fn database() -> PgPool {
        let options = db_options().expect("Failed to create test database options");
        PgConnectionBuilder::create_pool_lazy(options, 1, 1)
    }
}
