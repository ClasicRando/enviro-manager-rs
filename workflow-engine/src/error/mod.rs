#[derive(Debug)]
pub enum Error {
    Sql(sqlx::Error),
    CommitError(sqlx::Error),
    RollbackError { orig: sqlx::Error, new: sqlx::Error },
    JobNotReady,
    ExecutorInit(&'static str),
    ExitedTask,
    RmpEncode(rmp_serde::encode::Error),
    RmpDecode(rmp_serde::decode::Error),
    Reqwest(reqwest::Error),
    Generic(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Sql(error)
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(error: rmp_serde::encode::Error) -> Self {
        Self::RmpEncode(error)
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(error: rmp_serde::decode::Error) -> Self {
        Self::RmpDecode(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Generic(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(error) => write!(f, "SQL Error\n{}", error),
            Self::CommitError(error) => {
                write!(f, "SQL Error during transaction commit\n{}", error)
            }
            Self::RollbackError { orig, new } => {
                write!(
                    f,
                    "SQL Error during transaction rollback\n{}\nOriginal Error\n{}",
                    new, orig
                )
            }
            Self::JobNotReady => write!(f, "Attempted to execute a job that is not ready yet"),
            Self::ExecutorInit(message) => {
                write!(
                    f,
                    "Error during executor initialization. {}. Exiting executor",
                    message
                )
            }
            Self::ExitedTask => write!(f, "Exited remote task run unexpectedly"),
            Self::RmpEncode(error) => {
                write!(f, "Encountered rmp encode error\n{}", error)
            }
            Self::RmpDecode(error) => {
                write!(f, "Encountered rmp decode error\n{}", error)
            }
            Self::Reqwest(error) => {
                write!(f, "Reqwest Error\n{}", error)
            }
            Self::Generic(error) => write!(f, "Generic error\n{}", error),
        }
    }
}

impl std::error::Error for Error {}
