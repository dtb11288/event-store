use crate::event::Event;

#[async_trait::async_trait]
pub trait Command {
    type Context;
    type Event: Event;
    type Error;
    async fn handle_by(self, context: &mut Self::Context, state: Option<<<Self as Command>::Event as Event>::State>) -> Result<Vec<Self::Event>, Self::Error>;
}
