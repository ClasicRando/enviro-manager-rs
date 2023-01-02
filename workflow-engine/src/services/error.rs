use crate::database::TransactionError;

use super::jobs::JobError;

pub enum ServicesError {
    Sql(sqlx::Error),
    Transaction(TransactionError),
    Job(JobError)
}

pub type ServiceResult<T> = Result<T, ServicesError>;

impl From<sqlx::Error> for ServicesError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sql(error)
    }
}

impl From<TransactionError> for ServicesError {
    fn from(error: TransactionError) -> Self {
        Self::Transaction(error)
    }
}

impl From<JobError> for ServicesError {
    fn from(error: JobError) -> Self {
        Self::Job(error)
    }
}
