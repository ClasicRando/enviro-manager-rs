use std::env;

use lettre::{
    transport::smtp::{authentication::Credentials, response::Response},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use log::info;

use crate::error::EmResult;

/// Service to enable sending email alerts to desired targets
pub trait EmailService {
    /// Email sent response type
    type Response;
    /// Send an email to the desired recipient, with the provided `subject` and `body`
    /// # Errors
    /// This function will return an error if an error is returned creating the email message or
    /// sending the email.
    async fn send_email<S>(&self, to: S, subject: S, body: S) -> EmResult<Self::Response>
    where
        S: AsRef<str>;
}

/// Default implementation of an [EmailService]
pub struct ClippyEmailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl ClippyEmailService {
    /// Create a new instance of [ClippyEmailService]. Reads environment variables to capture
    /// details required to send emails as clippy.
    /// # Errors
    /// This function will returns an error if there are missing environment variables or the SMTP
    /// transport cannot be created. Require environment variables are:
    /// - CLIPPY_USERNAME -> email service username
    /// - CLIPPY_PASSWORD -> email service password
    /// - CLIPPY_RELAY -> email service relay
    pub fn new() -> EmResult<Self> {
        let username = env::var("CLIPPY_USERNAME")?;
        let password = env::var("CLIPPY_PASSWORD")?;
        let relay = env::var("CLIPPY_RELAY")?;
        let credentials = Credentials::from((username, password));
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&relay)?
            .credentials(credentials)
            .build();
        Ok(Self { mailer })
    }
}

impl EmailService for ClippyEmailService {
    type Response = Response;

    async fn send_email<S>(&self, to: S, subject: S, body: S) -> EmResult<Self::Response>
    where
        S: AsRef<str>,
    {
        info!(
            "Sending error email to {} with message\n{}",
            to.as_ref(),
            body.as_ref()
        );
        let email = Message::builder()
            .from("Clippy".parse()?)
            .to(to.as_ref().parse()?)
            .subject(subject.as_ref())
            .body(body.as_ref().to_owned())?;
        let response = self.mailer.send(email).await?;
        Ok(response)
    }
}
