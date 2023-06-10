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

/// Container for multiple optional errors that could arise from an execution of an anonymous block
/// of sql code with a rolled back transaction.
pub struct RolledBackTransactionResult<B, T>
where
    B: std::error::Error,
    T: std::error::Error,
{
    /// Error within the original block executed
    pub block_error: Option<B>,
    /// Error during the transaction rollback
    pub transaction_error: Option<T>,
}

impl<B, T> Default for RolledBackTransactionResult<B, T>
where
    B: std::error::Error,
    T: std::error::Error,
{
    fn default() -> Self {
        Self {
            block_error: None,
            transaction_error: None,
        }
    }
}

impl<B, T> RolledBackTransactionResult<B, T>
where
    B: std::error::Error,
    T: std::error::Error,
{
    /// Add or replace the current `block_error` attribute with `error`
    fn with_block_error(&mut self, error: B) {
        self.block_error = Some(error);
    }

    /// Add or replace the current `transaction_error` attribute with `error`
    fn with_transaction_error(&mut self, error: T) {
        self.transaction_error = Some(error);
    }
}
