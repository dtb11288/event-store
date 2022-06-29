use async_trait::async_trait;
use crate::prelude::Event;
use crate::{EventInfo, Id, StateInfo};

#[async_trait]
pub trait EventStore {
    type Error;
    async fn save<E: Event + Send + Sync>(&mut self, event: &EventInfo<E>) -> Result<Id, Self::Error>;
    async fn find_by_id<E: Event>(&self, id: &Id) -> Result<StateInfo<<E as Event>::State>, Self::Error>;
    // async fn find_by_version<E: Event>(&self, id: &Id, version: i64) -> Result<StateInfo<<E as Event>::State>, Self::Error>;
}

