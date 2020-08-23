mod info;
mod event;
mod state;
mod command;

pub mod prelude {
    pub use crate::event::EventType;
    pub use crate::event::Event;
    pub use crate::state::State;
    pub use crate::command::Command;
}
pub use self::info::*;
pub use self::event::EventInfo;
pub use self::state::StateInfo;

#[cfg(test)]
mod tests {
    use serde::{Serialize, Deserialize};
    use crate::prelude::*;
    use crate::tests::UserEvent::{AddUser, RenameUser};
    use crate::UserType;

    #[derive(Debug, Serialize, Clone, PartialEq)]
    struct User {
        pub email: String,
        pub name: String,
    }

    impl State for User {}

    #[derive(Debug, Serialize, Deserialize, Clone)]
    enum UserEvent {
        #[serde(rename = "add_user")]
        AddUser { email: String, name: String },
        #[serde(rename = "rename_user")]
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
            Ok(vec![self.clone()])
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
}
