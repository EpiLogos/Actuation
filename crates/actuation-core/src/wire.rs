use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{collections::BTreeMap, fmt};

pub type Result<T> = std::result::Result<T, Error>;
pub type Extensions = BTreeMap<String, Value>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error(String);
impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for Error {}
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

/// Optional wire facts are not collapsed into `Option`: absence is not null.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Slot<T> {
    #[default]
    Absent,
    Null,
    Value(T),
}
impl<T> Slot<T> {
    pub fn is_absent(&self) -> bool {
        matches!(self, Self::Absent)
    }
    pub fn value(&self) -> Option<&T> {
        if let Self::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub fn into_value(self) -> Option<T> {
        if let Self::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }
}
impl<T> From<T> for Slot<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}
impl<T: Serialize> Serialize for Slot<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Value(value) => value.serialize(serializer),
            _ => serializer.serialize_none(),
        }
    }
}
impl<'de, T: DeserializeOwned> Deserialize<'de> for Slot<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.is_null() {
            Ok(Self::Null)
        } else {
            serde_json::from_value(value)
                .map(Self::Value)
                .map_err(serde::de::Error::custom)
        }
    }
}

/// An admitted sequence has at least one member. Order and duplicates are not
/// silently rewritten; uniqueness is a separate relation-specific invariant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct NonEmpty<T>(Vec<T>);
impl<T> NonEmpty<T> {
    pub fn new(items: Vec<T>) -> Result<Self> {
        if items.is_empty() {
            Err(Error::new("sequence must not be empty"))
        } else {
            Ok(Self(items))
        }
    }
    pub fn one(item: T) -> Self {
        Self(vec![item])
    }
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
    pub fn first(&self) -> &T {
        &self.0[0]
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for NonEmpty<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Self::new(Vec::<T>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Typed drafts implement their own semantic invariant. Generic admission
/// also prevents extension keys from shadowing declared identity/authority.
pub trait Invariant {
    const FIELDS: &'static [&'static str];
    fn extensions(&self) -> &Extensions;
    fn check(&self) -> Result<()>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record<T>(T);
impl<T: Invariant> Record<T> {
    pub fn new(fields: T) -> Result<Self> {
        if let Some(name) = fields
            .extensions()
            .keys()
            .find(|name| T::FIELDS.contains(&name.as_str()))
        {
            return Err(Error::new(format!(
                "extension cannot shadow declared field {name}"
            )));
        }
        fields.check()?;
        Ok(Self(fields))
    }
    pub fn fields(&self) -> &T {
        &self.0
    }
    pub fn into_fields(self) -> T {
        self.0
    }
}
impl<T: Serialize> Serialize for Record<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de, T: Invariant + DeserializeOwned> Deserialize<'de> for Record<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom("record must be an object"));
        }
        let fields = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Self::new(fields).map_err(serde::de::Error::custom)
    }
}
impl<T: Invariant + DeserializeOwned> TryFrom<Value> for Record<T> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        serde_json::from_value(value).map_err(Into::into)
    }
}
