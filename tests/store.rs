use std::collections::HashMap;
use event_store::{EventInfo, Id, UserType, Info, StateInfo};
use event_store::prelude::*;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use event_derive::*;

#[derive(Debug, PartialEq, Eq)]
struct TestStore {
    data: HashMap<String, Vec<Value>>,
}

#[async_trait]
impl EventStore for TestStore {
    type Error = ();

    async fn save<E: Event + Send + Sync + Serialize>(&mut self, event: &EventInfo<E>) -> Result<Id, Self::Error> {
        let id = event.info().id.to_string();
        if let Some(mut old_events) = self.data.remove(&id) {
            old_events.push(serde_json::to_value(event).unwrap());
            self.data.insert(id.clone(), old_events);
        } else {
            self.data.insert(id.clone(), vec![serde_json::to_value(event).unwrap()]);
        }
        Ok(event.info().id.clone())
    }

    async fn find_by_id<E: Event>(&self, id: &Id) -> Result<StateInfo<<E as Event>::State>, Self::Error> {
        let data = self.data.get(&id.to_string());
        let state = if let Some(values) = data {
            let events = values.iter().map(|value| {
                let event: EventInfo<E> = serde_json::from_value(value.clone()).unwrap();
                event
            });
            <E as Event>::State::init().replay(events)
        } else {
            <E as Event>::State::init()
        };
        Ok(state)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, State, PartialEq)]
pub struct User {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, EventType, Clone, PartialEq, Eq)]
#[serde(tag = "event", content = "data")]
#[stream = "user"]
pub enum UserEvent {
    #[serde(rename = "user_added")]
    AddUser { email: String, name: String },
    #[serde(rename = "user_renamed")]
    RenameUser(String),
}

impl Event for UserEvent {
    type State = User;
    fn apply_to(self, state: Option<Self::State>) -> Self::State {
        match self {
            UserEvent::AddUser { name, email } => User {
                email,
                name,
            },
            UserEvent::RenameUser(new_name) => {
                let mut user = state.unwrap();
                user.name = new_name;
                user
            }
        }
    }
}

#[tokio::test]
async fn event_store_save_events() {
    let mut store = TestStore { data: HashMap::new() };
    let info = Info::new::<UserEvent>(UserType::guest());
    let user_added = EventInfo::new(info.clone(), UserEvent::AddUser { email: "test@gmail.com".to_string(), name: "Binh".to_string() });
    let user_id = info.id.clone();
    let info = info.increase(UserType::user(user_id));
    let user_renamed = EventInfo::new(info.clone(), UserEvent::RenameUser("Biz8".to_owned()));

    store.save(&user_added).await.unwrap();
    let id = store.save(&user_renamed).await.unwrap();
    assert_eq!(store.data.get(&id.to_string()).unwrap().len(), 2);

    let user = User {
        email: "test@gmail.com".to_string(),
        name: "Biz8".to_string()
    };
    let saved_user = store.find_by_id::<UserEvent>(&id)
        .await
        .unwrap();
    let saved_user = saved_user.data().unwrap().clone();
    assert_eq!(saved_user, user);
}
