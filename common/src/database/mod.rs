use crate::error::EmResult;

pub mod build;
pub mod connection;
pub mod listener;
pub mod postgres;
pub mod test;

/// Describes the high level abilities of the database operated against. Must
pub trait Database {
    /// Options to allow for connecting to the database
    type ConnectionOptions;
    /// Type for holding a pool of connections to the database
    type ConnectionPool: Clone;
    /// Return a new pool of database connections
    async fn create_pool(
        options: Self::ConnectionOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> EmResult<Self::ConnectionPool>;
    /// Return a new pool of database connections with connections not explicitly created
    fn create_pool_lazy(
        options: Self::ConnectionOptions,
        max_connections: u32,
        min_connection: u32,
    ) -> Self::ConnectionPool;
}
