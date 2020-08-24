use serde::{Serialize, Deserialize};
use crate::info::Info;
use crate::state::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInfo<E> {
    #[serde(flatten)]
    info: Info,
    #[serde(flatten)]
    payload: E,
}

pub trait EventType {
    fn stream_type() -> &'static str;
}

pub trait Event: EventType + Serialize + for<'de> Deserialize<'de> {
    type State: State;
    fn apply_to(self, state: Option<Self::State>) -> Self::State;
}

impl<E: Event> EventInfo<E> {
    pub fn new(info: Info, payload: E) -> Self {
        Self {
            info,
            payload,
        }
    }

    pub fn take(self) -> (Info, E) {
        (self.info, self.payload)
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn event(&self) -> &E {
        &self.payload
    }
}
