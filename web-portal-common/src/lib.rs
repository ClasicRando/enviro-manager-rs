use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub full_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: Uuid,
}
