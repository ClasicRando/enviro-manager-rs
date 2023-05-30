use crate::database::{Database, RolledBackTransactionResult};

///
pub trait DatabaseTester
where
    Self: Send + Sync,
{
    ///
    type BlockError: std::error::Error;
    ///
    type Database: Database;
    ///
    type TransactionError: std::error::Error;
    ///
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
