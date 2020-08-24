use crate::event::EventType;
use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};
use uuid::Uuid;
use uuid::parser::ParseError;
use core::fmt::{Display, Formatter, Error};
use core::str::FromStr;
use core::ops::Deref;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Id(Uuid);

impl Id {
    pub fn random() -> Self {
        Id(Uuid::new_v4())
    }
}

impl Deref for Id {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Id {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s)
            .map(Self)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemUser {
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "guest")]
    Guest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserType {
    System(SystemUser),
    User(Id),
}

impl UserType {
    pub fn guest() -> Self {
        UserType::System(SystemUser::Guest)
    }

    pub fn root() -> Self {
        UserType::System(SystemUser::Root)
    }

    pub fn user(id: Id) -> Self {
        UserType::User(id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Info {
    pub id: Id,
    pub stream: String,
    pub date: DateTime<Utc>,
    pub user: UserType,
    pub version: i64,
}

impl Info {
    pub fn new<E: EventType>(user: UserType) -> Self {
        Self {
            id: Id::random(),
            stream: E::stream_type().to_owned(),
            date: Utc::now(),
            version: 1,
            user,
        }
    }

    pub fn date(mut self, date: DateTime<Utc>) -> Self {
        self.date = date;
        self
    }

    pub fn increase(self, user: UserType) -> Info {
        Self {
            user,
            date: Utc::now(),
            version: self.version + 1,
            ..self
        }
    }
}
