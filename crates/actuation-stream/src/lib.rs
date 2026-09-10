//! Attributable actuality, not an activity planner or a provider host.
//!
//! Stream/event identity, observation standing, and terminal state are admitted
//! before mutation. The append-only JSONL format remains the public store.
//! Correlation consumes supplied owner facts; a missing side is not an empty one.
mod activity;
mod correlation;
mod event;
mod journal;
mod store;
mod stream;
mod usage;
mod wire;
pub use activity::*;
pub use actuation_core::{Error, Result};
pub use correlation::*;
pub use event::*;
pub use journal::*;
pub use store::*;
pub use stream::*;
pub use usage::*;
pub use wire::{Count, JsonObject, Timestamp};

macro_rules! domain {
    ($name:ident, $fields:ident, [$($field:ident),*], $check:expr) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(actuation_core::Record<$fields>);
        impl $name {
            pub fn new(fields: $fields) -> actuation_core::Result<Self> {
                Ok(Self(actuation_core::Record::new(fields)?))
            }
            pub fn fields(&self) -> &$fields { self.0.fields() }
            pub fn into_fields(self) -> $fields { self.0.into_fields() }
        }
        impl TryFrom<serde_json::Value> for $name {
            type Error = actuation_core::Error;
            fn try_from(value: serde_json::Value) -> actuation_core::Result<Self> {
                serde_json::from_value(value).map_err(Into::into)
            }
        }
        impl actuation_core::Invariant for $fields {
            const FIELDS: &'static [&'static str] = &[$(stringify!($field)),*];
            fn extensions(&self) -> &actuation_core::Extensions { &self.extensions }
            fn check(&self) -> actuation_core::Result<()> { ($check)(self) }
        }
    };
}
pub(crate) use domain;
