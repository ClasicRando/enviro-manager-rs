use common::api::ApiResponseBody;
use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use web_portal_common::{Credentials, Session, User};

#[derive(Clone, Copy)]
pub struct UnauthorizedApi {
    url: &'static str,
}

impl UnauthorizedApi {
    pub fn new(url: &'static str) -> Self {
        Self { url }
    }

    pub async fn login(&self, credentials: &Credentials) -> Result<AuthorizedApi> {
        let url = format!("{}/login", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        let ApiSuccess::Data(session) = parse_response::<Session>(response).await? else {
            return Err(Error::UnexpectedMessage)
        };
        Ok(AuthorizedApi::new(self.url, format!("{}", session.token)))
    }
}

#[derive(Clone)]
pub struct AuthorizedApi {
    url: &'static str,
    token: String,
}

impl AuthorizedApi {
    pub fn new(url: &'static str, token: String) -> Self {
        Self { url, token }
    }

    pub async fn logout(&self) -> Result<()> {
        let url = format!("{}/logout", self.url);
        self.send::<()>(Request::post(&url)).await?;
        Ok(())
    }

    pub async fn user_info(&self) -> Result<User> {
        let url = format!("{}/user", self.url);
        let user = match self.send::<User>(Request::get(&url)).await? {
            ApiSuccess::Data(user) => user,
            ApiSuccess::Message(message) => {
                log::debug!("{}", message);
                return Err(Error::UnexpectedMessage);
            }
        };
        Ok(user)
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token)
    }

    async fn send<T>(&self, req: Request) -> Result<ApiSuccess<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let response = req
            .header("Authorization", &self.auth_header_value())
            .send()
            .await?;
        parse_response(response).await
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Fetch(#[from] gloo_net::Error),
    #[error("API error\n{0}")]
    ApiError(String),
    #[error("Expecting data but got message")]
    UnexpectedMessage,
    #[error("Expecting message but got data")]
    UnexpectedData,
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::ApiError(value.to_owned())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::ApiError(value)
    }
}

pub enum ApiSuccess<T> {
    Data(T),
    Message(String),
}

async fn parse_response<T>(response: Response) -> Result<ApiSuccess<T>>
where
    T: Serialize + DeserializeOwned,
{
    // ensure we've got 2xx status
    if response.ok() {
        match response.json::<ApiResponseBody<T>>().await? {
            ApiResponseBody::Success(data) => Ok(ApiSuccess::Data(data)),
            ApiResponseBody::Message(message) => Ok(ApiSuccess::Message(message)),
            ApiResponseBody::Failure(message) | ApiResponseBody::Error(message) => {
                Err(message.into())
            }
        }
    } else {
        Err(response.text().await?.into())
    }
}
