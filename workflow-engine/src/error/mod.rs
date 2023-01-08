use chrono::NaiveDateTime;
use lettre::{
    address::AddressError, error::Error as EmailError, transport::smtp::Error as StmpError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Generic SQL error\n{0}")]
    Sql(#[from] sqlx::Error),
    #[error("SQL Error during transaction commit\n{0}")]
    CommitError(sqlx::Error),
    #[error("SQL Error during transaction rollback\n{orig}\nOriginal Error\n{new}")]
    RollbackError { orig: sqlx::Error, new: sqlx::Error },
    #[error("Job attempted to start before next run")]
    JobNotReady,
    #[error("Error during executor initialization. {0}. Exiting executor")]
    ExecutorInit(&'static str),
    #[error("Exited remote task run unexpectedly")]
    ExitedTask,
    #[error("MessagePack encode error\n{0}")]
    RmpEncode(#[from] rmp_serde::encode::Error),
    #[error("MessagePack decode error\n{0}")]
    RmpDecode(#[from] rmp_serde::decode::Error),
    #[error("Reqwest Error\n{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Generic error\n{0}")]
    Generic(String),
    #[error("Duplicate Job Entry\nId: {0}\nNext Runs:{1:?}")]
    DuplicateJobId(i64, [NaiveDateTime; 2]),
    #[error("Notification Payload Parse Error\nNotification: `{0}`")]
    PayloadParseError(String),
    #[error("Reqwest Error\n{0}")]
    Lettre(#[from] EmailError),
    #[error("Reqwest Error\n{0}")]
    AddressParseError(#[from] AddressError),
    #[error("Reqwest Error\n{0}")]
    SmtpError(#[from] StmpError),
}

pub type Result<T> = std::result::Result<T, Error>;
