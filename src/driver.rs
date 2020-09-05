use crate::store::EventStore;
use crate::bus::EventBus;
use crate::prelude::Event;
use crate::{EventInfo, Id};
use futures::Stream;

pub struct EventDriver<S, B> {
    store: S,
    bus: B,
}

impl<Err, S: EventStore<Error = Err>, B: EventBus<Error = Err>> EventDriver<S, B> {
    pub fn new(store: S, bus: B) -> Self {
        Self {
            store,
            bus,
        }
    }

    pub async fn append<E: Event + Send + Sync>(&mut self, event: EventInfo<E>) -> Result<Id, Err> {
        let id = self.store.save(&event).await?;
        self.bus.publish(&event).await?;
        Ok(id)
    }

    pub async fn register<'a, E: Event + 'a>(&'a mut self) -> Result<impl Stream + 'a, Err> {
        self.bus.register::<E>().await
    }
}
