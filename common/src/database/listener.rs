use crate::{database::Database, error::EmResult};

/// State change listener.
pub trait ChangeListener
where
    Self: Send + Sync,
{
    /// Database that is checked by this listener
    type Database: Database;
    /// Message type that is piped through the listener
    type Message;
    /// Receive the next message from the listen channel
    async fn recv(&mut self) -> EmResult<Self::Message>;
}
