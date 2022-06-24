use async_trait::async_trait;
use futures::stream::BoxStream;
use crate::prelude::Event;
use crate::EventInfo;

#[async_trait]
pub trait EventBus {
    type Error;
    async fn publish<E: Event + Send + Sync>(&mut self, event: EventInfo<E>) -> Result<(), Self::Error>;
    async fn register<E: Event>(&mut self) -> Result<BoxStream<'_, EventInfo<E>>, Self::Error>;
}
