use crate::event::Event;

#[async_trait::async_trait]
pub trait Command<C> {
    type Event: Event;
    type Error;
    async fn handle_by(self, handler: C, state: Option<<<Self as Command<C>>::Event as Event>::State>) -> Result<Vec<Self::Event>, Self::Error>;
}
