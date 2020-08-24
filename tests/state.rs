use event_store::prelude::*;
use crate::UserEvent::{AddUser, RenameUser};
use event_store::UserType;
use serde::{Serialize, Deserialize};
use event_derive::*;

#[derive(Debug, Serialize, Clone, State, PartialEq)]
struct User {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum UserEvent {
    AddUser { email: String, name: String },
    RenameUser { new_name: String },
}

impl EventType for UserEvent {
    fn stream_type() -> &'static str {
        "user"
    }
}

impl Event for UserEvent {
    type State = User;
    fn apply_to(self, state: Option<Self::State>) -> Self::State {
        match self {
            UserEvent::AddUser { name, email } => User {
                email: email.clone(),
                name: name.clone(),
            },
            UserEvent::RenameUser { new_name } => {
                let mut user = state.unwrap().clone();
                user.name = new_name.clone();
                user
            }
        }
    }
}

impl Command for UserEvent {
    type Event = UserEvent;
    type Error = ();

    fn handle_by(self, _state: Option<User>) -> Result<Vec<UserEvent>, ()> {
        Ok(vec![self])
    }
}

#[test]
fn generate_state_by_events() {
    let user_init = User::init();
    let add_user = user_init.clone().handle(AddUser {
        email: "demo@my.com".to_string(),
        name: "name1".to_string(),
    }, UserType::guest())
        .unwrap()
        .first()
        .unwrap()
        .clone();

    let user = user_init + add_user.clone();
    assert_eq!(1, user.info().unwrap().version);
    assert_eq!("name1", user.data().unwrap().name);

    let rename_user = user.clone().handle(RenameUser { new_name: "name2".to_string() }, UserType::root())
        .unwrap()
        .first()
        .unwrap()
        .clone();

    let user = user + rename_user.clone();
    assert_eq!(2, user.info().unwrap().version);
    assert_eq!("name2", user.data().unwrap().name);
    assert_eq!(user, User::init() + add_user + rename_user);
}
