use crate::database::{Database, RolledBackTransactionResult};

/// Behaviour to allow for databases to have various test scripts run. This type should be
/// implemented for every [Database] implementation once and utilized in a database specific
/// function.
pub trait DatabaseTester
where
    Self: Send + Sync,
{
    /// Error type that is returned when an anonymous block returns an error
    type BlockError: std::error::Error;
    /// Database variation that have the test scripts executed against
    type Database: Database;
    /// Error type that is returned when a transaction rollback fails
    type TransactionError: std::error::Error;
    /// Create a new instance of the [DatabaseTester] using the [Database]'s connection `pool`
    fn create(pool: &<Self::Database as Database>::ConnectionPool) -> Self;
    /// Execute the provided `block` of database code against the `pool`. If the block does not
    /// match the required formatting to be an anonymous block, the code is wrapped in the required
    /// code to ensure the execution can be completed. The entire block is executed within a rolled
    /// back transaction, returning the errors of the block and transaction rollback, if any,
    /// respectively within a tuple.
    async fn execute_anonymous_block_transaction(
        &self,
        block: &str,
    ) -> RolledBackTransactionResult<Self::BlockError, Self::TransactionError>;
}
