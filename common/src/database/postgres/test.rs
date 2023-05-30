use sqlx::PgPool;

use crate::database::{
    postgres::{format_anonymous_block, Postgres},
    test::DatabaseTester,
    Database, RolledBackTransactionResult,
};

pub struct PgDatabaseTester {
    pool: PgPool,
}

impl DatabaseTester for PgDatabaseTester {
    type BlockError = sqlx::Error;
    type Database = Postgres;
    type TransactionError = sqlx::Error;

    fn create(pool: &<Self::Database as Database>::ConnectionPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn execute_anonymous_block_transaction(
        &self,
        block: &str,
    ) -> RolledBackTransactionResult<Self::BlockError, Self::TransactionError> {
        let block = format_anonymous_block(block);
        let mut result = RolledBackTransactionResult::default();
        let mut transaction = match self.pool.begin().await {
            Ok(inner) => inner,
            Err(error) => {
                result.with_transaction_error(error);
                return result;
            }
        };
        if let Err(error) = sqlx::query(&block).execute(&mut transaction).await {
            result.with_block_error(error);
        }
        if let Err(error) = transaction.rollback().await {
            result.with_transaction_error(error);
        }
        result
    }
}
