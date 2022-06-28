use crate::command::{Command, CommandHandler};
use crate::event::{Event, EventInfo};
use crate::info::{Info, UserType};
use serde::Serialize;
use core::ops::Add;

#[derive(Debug, Clone, PartialEq, Serialize)]
struct InnerState<T> {
    #[serde(flatten)]
    info: Info,
    data: T,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateInfo<T>(Option<InnerState<T>>);

pub trait State: Sized {
    fn init() -> StateInfo<Self> { StateInfo(None) }
}

impl<E: Event> Add<EventInfo<E>> for StateInfo<<E as Event>::State> {
    type Output = Self;

    fn add(self, rhs: EventInfo<E>) -> Self::Output {
        self.apply(rhs)
    }
}

impl<T: State> StateInfo<T> {
    pub fn info(&self) -> Option<&Info> {
        self.0.as_ref().map(|state| &state.info)
    }

    pub fn data(&self) -> Option<&T> {
        self.0.as_ref().map(|state| &state.data)
    }

    fn take(self) -> (Option<Info>, Option<T>) {
        if let Some(state) = self.0 {
            (Some(state.info), Some(state.data))
        } else {
            (None, None)
        }
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn handle<E: Event<State = T>, Err>(self, command: impl Command<Event=E, Error=Err>, user: UserType) -> Result<Vec<EventInfo<E>>, Err> {
        let (info, data) = self.take();
        command.handle_by(data)
            .map(|events| {
                let (_, events) = events.into_iter()
                    .fold((info, vec![]), |(info, mut events), event| {
                        let user = (&user).clone();
                        let info = if let Some(info) = info {
                            info.increase(user)
                        } else {
                            Info::new::<E>(user)
                        };
                        let event = EventInfo::new(info.clone(), event);
                        events.push(event);
                        (Some(info), events)
                    });
                events
            })
    }

    pub fn apply<E: Event<State = T>>(self, e: EventInfo<E>) -> Self {
        let (info, event) = e.take();
        let data = event.apply_to(self.take().1);
        Self(Some(InnerState { info, data }))
    }

    pub fn replay<E: Event<State = T>>(self, events: impl Iterator<Item=EventInfo<E>>) -> Self {
        events.fold(self, Self::apply)
    }
}
