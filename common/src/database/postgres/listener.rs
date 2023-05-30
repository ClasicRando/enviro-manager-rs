use std::{marker::PhantomData, str::FromStr};

use log::error;
use sqlx::postgres::PgListener;

use crate::{
    database::{listener::ChangeListener, postgres::Postgres},
    error::{EmError, EmResult},
};

pub struct PgChangeListener<M>
where
    M: FromStr<Err = EmError> + Send + Sync,
{
    listener: PgListener,
    marker: PhantomData<M>,
}

impl<M> PgChangeListener<M>
where
    M: FromStr<Err = EmError> + Send + Sync,
{
    pub fn new(listener: PgListener) -> Self {
        Self {
            listener,
            marker: PhantomData,
        }
    }
}

impl<M> ChangeListener for PgChangeListener<M>
where
    M: FromStr<Err = EmError> + Send + Sync,
{
    type Database = Postgres;
    type Message = M;

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
