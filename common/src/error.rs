use chrono::NaiveDateTime;
use lettre::{
    address::AddressError, error::Error as EmailError, transport::smtp::Error as StmpError,
};
use sqlx::types::Uuid;
use thiserror::Error;

use crate::api::ApiRequest;

/// All possible error types that may occur during workflow engine operations
#[derive(Error, Debug)]
pub enum EmError {
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
    #[error("Json serde error\n{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Reqwest Error\n{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Generic error\n{0}")]
    Generic(String),
    #[error("Duplicate Job Entry\nId: {0}\nNext Runs:{1:?}")]
    DuplicateJobId(i64, [NaiveDateTime; 2]),
    #[error("Notification Payload Parse Error\nNotification: `{0}`")]
    PayloadParseError(String),
    #[error("Email Error\n{0}")]
    Lettre(#[from] EmailError),
    #[error("Email Address Error\n{0}")]
    AddressParseError(#[from] AddressError),
    #[error("SMTP Error\n{0}")]
    SmtpError(#[from] StmpError),
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Environment Variable error\n{0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("IO error\n{0}")]
    IO(#[from] std::io::Error),
    #[error("Invalid User")]
    InvalidUser,
    #[error("User missing privilege. UID = {uid}, role = {role}")]
    MissingPrivilege { uid: Uuid, role: &'static str },
    #[error("Password is not valid. {reason}")]
    InvalidPassword { reason: &'static str },
    #[error("Record cannot be found for `{pk}`")]
    MissingRecord { pk: String },
    #[error("Contents of request '{request}' were not valid.\nReason: {reason}")]
    InvalidRequest { request: String, reason: String },
}

impl From<&str> for EmError {
    fn from(value: &str) -> Self {
        Self::Generic(value.to_string())
    }
}

impl From<String> for EmError {
    fn from(value: String) -> Self {
        Self::Generic(value)
    }
}

impl<R> From<(&R, String)> for EmError
where
    R: ApiRequest,
{
    fn from(value: (&R, String)) -> Self {
        Self::InvalidRequest {
            request: format!("{:?}", value.0),
            reason: value.1,
        }
    }
}

/// Generic [Result] type where the error is always [Error]
pub type EmResult<T> = Result<T, EmError>;
