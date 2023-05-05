use std::{marker::PhantomData, str::FromStr};

use common::error::{EmError, EmResult};
use log::error;
use sqlx::postgres::PgListener;

#[async_trait::async_trait]
pub trait ChangeListener<M: FromStr<Err = EmError>>: Send {
    async fn recv(&mut self) -> EmResult<M>;
}

pub struct PgChangeListener<M>
where
    M: FromStr<Err = EmError> + Send,
{
    listener: PgListener,
    marker: PhantomData<M>,
}

impl<M> PgChangeListener<M>
where
    M: FromStr<Err = EmError> + Send,
{
    pub fn new(listener: PgListener) -> Self {
        Self {
            listener,
            marker: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<M> ChangeListener<M> for PgChangeListener<M>
where
    M: FromStr<Err = EmError> + Send,
{
    async fn recv(&mut self) -> EmResult<M> {
        let notification = match self.listener.recv().await {
            Ok(notification) => notification,
            Err(error) => {
                error!("Error receiving notification.\n{:?}", error);
                return Err(error.into());
            }
        };
        notification.payload().parse()
    }
}
