use event_store::EventInfo;
use futures::stream::BoxStream;
use async_trait::async_trait;
use futures::StreamExt;
use event_store::prelude::*;
use crate::UserEvent::{AddUser, RenameUser};
use event_store::UserType;
use serde::{Serialize, Deserialize};
use event_derive::*;
use redis::{AsyncCommands, RedisError};
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

pub struct Bus {
    client: redis::Client,
    pubsub: HashMap<&'static str, redis::aio::PubSub>,
}

impl Bus {
    pub async fn new() -> Self {
        let rd = redis::Client::open("redis://127.0.0.1/").unwrap();
        Self {
            client: rd,
            pubsub: HashMap::new(),
        }
    }
}

#[async_trait]
impl EventBus for Bus {
    type Error = RedisError;

    async fn publish<E: Event + Send + Sync>(&self, event: &EventInfo<E>) -> Result<(), Self::Error> {
        let mut publish_conn = self.client.get_async_connection().await?;
        publish_conn.publish(E::stream_type(), serde_json::to_string(event).unwrap())
            .await
            .map(|_: redis::Value| ())?;
        Ok(())
    }

    async fn register<E: Event>(&mut self) -> Result<BoxStream<'_, EventInfo<E>>, Self::Error> {
        self.pubsub.insert(E::stream_type(), self.client.get_async_connection().await?.into_pubsub());
        let pubsub_conn = self.pubsub.get_mut(E::stream_type()).unwrap();
        pubsub_conn.subscribe(E::stream_type()).await?;
        let stream = pubsub_conn.on_message()
            .map(|msg| {
                let payload: String = msg.get_payload().unwrap();
                let event: EventInfo<E> = serde_json::from_str(payload.as_str()).unwrap();
                event
            })
            .boxed();
        Ok(stream)
    }
}

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

#[tokio::test]
async fn test_bus() {
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

    let rename_user = user.clone().handle(RenameUser { new_name: "name2".to_string() }, UserType::root())
        .unwrap()
        .first()
        .unwrap()
        .clone();

    let bus = Arc::new(Mutex::new(Bus::new().await));
    {
        let bus = bus.clone();
        std::thread::spawn(move || {
            futures::executor::block_on(async {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let bus = bus.lock().unwrap();
                bus.publish::<UserEvent>(&add_user).await.unwrap();
                bus.publish::<UserEvent>(&rename_user).await.unwrap();
            });
        }).join().unwrap();
    }

    std::thread::spawn(move || {
        futures::executor::block_on(async {
            let mut bus = bus.lock().unwrap();
            let mut stream = bus.register::<UserEvent>().await.unwrap();
            while let Some(event) = stream.next().await {
                dbg!(event);
            }
        });
    }).join().unwrap();

    // let mut bus_str = bus.as_ref().borrow_mut();
    // let mut stream = bus_str.register::<UserEvent>().await.unwrap();
    // bus_str.publish(&add_user).await.unwrap();
    // bus_str.publish(&rename_user).await.unwrap();
    // let msg: EventInfo<UserEvent> = stream.next().await.unwrap();
    // dbg!(msg);
}

#[tokio::test]
async fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut publish_conn = client.get_async_connection().await?;
    let mut pubsub_conn = client.get_async_connection().await?.into_pubsub();

    pubsub_conn.subscribe("wavephone").await?;
    let mut pubsub_stream = pubsub_conn.on_message();

    publish_conn.publish("wavephone", "banana").await?;

    let pubsub_msg: String = pubsub_stream.next().await.unwrap().get_payload()?;
    assert_eq!(&pubsub_msg, "banana");

    Ok(())
}
