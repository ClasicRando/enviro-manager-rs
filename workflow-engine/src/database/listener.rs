use std::{marker::PhantomData, str::FromStr};

use common::error::{EmError, EmResult};
use log::error;
use sqlx::postgres::PgListener;

pub trait ChangeListener {
    type Message;
    async fn recv(&mut self) -> EmResult<Self::Message>;
}

pub struct PgChangeListener<M> {
    listener: PgListener,
    marker: PhantomData<M>,
}

impl<M> PgChangeListener<M> {
    pub fn new(listener: PgListener) -> Self {
        Self {
            listener,
            marker: PhantomData,
        }
    }
}

impl<M> ChangeListener for PgChangeListener<M>
where
    M: FromStr<Err = EmError>,
{
    type Message = M;

    async fn recv(&mut self) -> EmResult<Self::Message> {
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
