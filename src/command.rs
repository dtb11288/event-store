use crate::event::Event;

pub trait Command {
    type Event: Event;
    type Error;
    fn handle_by(self, state: Option<<<Self as Command>::Event as Event>::State>) -> Result<Vec<Self::Event>, Self::Error>;
}
