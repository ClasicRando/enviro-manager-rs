use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{de::Visitor, Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;

/// Status of an [Executor][crate::executor::Executor] as found in the database as a simple
/// Postgresql enum type
#[derive(sqlx::Type, Serialize, PartialEq, Debug)]
#[sqlx(type_name = "executor_status")]
pub enum ExecutorStatus {
    Active,
    Canceled,
    Shutdown,
}

/// Method of deserializing an [IpNetwork] type
fn deserialize_ipnetwork<'de, D>(deserializer: D) -> Result<IpNetwork, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct IpNetworkVisitor;

    impl<'de> Visitor<'de> for IpNetworkVisitor {
        type Value = IpNetwork;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string that represents an ip address")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            IpNetwork::from_str(v)
                .map_err(|e| E::custom(format!("Could not parse value into IpNetwork: {e}")))
        }
    }
    deserializer.deserialize_string(IpNetworkVisitor)
}

/// Method of serializing an [IpNetwork] type
fn serialize_ipnetwork<S>(addr: &IpNetwork, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.collect_str(addr)
}

/// Executor data type representing a row from `executor.v_executor`
#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Executor {
    pub executor_id: ExecutorId,
    pub pid: i32,
    pub username: String,
    pub application_name: String,
    #[serde(
        serialize_with = "serialize_ipnetwork",
        deserialize_with = "deserialize_ipnetwork"
    )]
    pub client_addr: IpNetwork,
    pub client_port: i32,
    pub exec_start: NaiveDateTime,
    #[sqlx(default)]
    pub session_active: bool,
    #[sqlx(rename = "wr_count")]
    pub workflow_run_count: i64,
}

/// Wrapper for an `executor_id` value. Made to ensure data passed as the id of an executor is
/// correct and not just any i64 value.
#[derive(sqlx::Type, Clone, Deserialize, Serialize)]
#[sqlx(transparent)]
pub struct ExecutorId(i64);

impl std::fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
