#[cfg(test)]
mod tests {
    use serde::{Serialize, Deserialize};
    use event_derive::*;
    use event_type::prelude::*;
    use event_type::UserType;

    #[derive(Debug, Serialize, Deserialize, Clone, State, PartialEq)]
    pub struct User {
        pub email: String,
        pub name: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, EventType, PartialEq, Eq)]
    #[stream = "user"]
    pub enum UserEvent {
        #[serde(rename = "add_user")]
        AddUser { email: String, name: String },
        #[serde(rename = "rename_user")]
        RenameUser { new_name: String },
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
    fn test() {
        let user_init = User::init();
        let add_user = user_init.clone().handle(UserEvent::AddUser {
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

        let rename_user = user.clone().handle(UserEvent::RenameUser { new_name: "name2".to_string() }, UserType::root())
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
