use std::marker::PhantomData;

use common::error::EmResult;
use sqlx::postgres::PgListener;

pub trait FromPayload
where
    Self: Sized,
{
    fn from_payload(payload: &str) -> Self;
}

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
        Self { listener, marker: PhantomData }
    }
}

impl<M> ChangeListener for PgChangeListener<M>
where
    M: FromPayload,
{
    type Message = M;

    async fn recv(&mut self) -> EmResult<Self::Message> {
        let notification = self.listener.recv().await?;
        Ok(M::from_payload(notification.payload()))
    }
}
