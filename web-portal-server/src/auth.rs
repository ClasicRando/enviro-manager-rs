use chrono::{Days, Utc};
use common::error::EmResult;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

static SECRET_KEY: OnceCell<String> = OnceCell::new();

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: Uuid,
    exp: usize,
}

fn secret_key() -> Result<&'static String, std::env::VarError> {
    SECRET_KEY.get_or_try_init(|| std::env::var("JWT_SECRET"))
}

pub fn create_token(uid: Uuid) -> EmResult<String> {
    let now = Utc::now()
        .checked_add_days(Days::new(7))
        .ok_or("Could not get an expiration date for jwt")?;
    let claims = Claims {
        sub: uid,
        exp: now.timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key()?.as_str().as_ref()),
    )?;
    Ok(token)
}

pub fn decode_token(token: &str) -> EmResult<Claims> {
    let token_data = decode(
        token,
        &DecodingKey::from_secret(secret_key()?.as_str().as_ref()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
