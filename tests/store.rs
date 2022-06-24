use event_store::{EventInfo, Id, UserType, Info};
use event_store::prelude::*;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use event_derive::*;

#[derive(Debug, PartialEq, Eq)]
struct TestStore {
    data: Vec<String>
}

#[async_trait]
impl EventStore for TestStore {
    type Error = ();

    async fn save<E: Event + Send + Sync + Serialize>(&mut self, event: &EventInfo<E>) -> Result<Id, Self::Error> {
        let id = event.info().id.clone();
        self.data.push(serde_json::to_string(event).unwrap());
        Ok(id)
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
    let mut store = TestStore { data: vec![] };
    let info = Info::new::<UserEvent>(UserType::guest());
    let user_added = EventInfo::new(info.clone(), UserEvent::AddUser { email: "test@gmail.com".to_string(), name: "Binh".to_string() });
    let info = info.increase(UserType::guest());
    let user_renamed = EventInfo::new(info.clone(), UserEvent::RenameUser("Biz8".to_owned()));

    store.save(&user_added).await.ok();
    store.save(&user_renamed).await.ok();
    assert_eq!(store.data.len(), 2);
}
