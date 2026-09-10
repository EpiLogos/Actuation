use crate::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};

// Preserve the public ECMAScript non-empty-ref rule, including BOM whitespace,
// without normalising the ref or treating every Unicode control as whitespace.
pub fn is_blank_reference(value: &str) -> bool {
    value.chars().all(|c| matches!(c, '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'))
}

macro_rules! references {
    ($($name:ident),+ $(,)?) => {$ (
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                if is_blank_reference(&value) { Err(Error::new(concat!(stringify!($name), " must be a non-empty string ref"))) }
                else { Ok(Self(value)) }
            }
            pub fn as_str(&self) -> &str { &self.0 }
            pub fn into_string(self) -> String { self.0 }
        }
        impl AsRef<str> for $name { fn as_ref(&self) -> &str { &self.0 } }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
        }
        impl FromStr for $name {
            type Err = Error;
            fn from_str(value: &str) -> Result<Self> { Self::new(value) }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
                Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    )+};
}

// Nominal distinctions prevent a Session, Run, provider or process reference
// from being passed where an enduring Agent/World-relative Agency is required.
// External refs have no resolution, inference or ownership behaviour here.
references!(
    AgentRef,
    AgencyRef,
    WorldBindingRef,
    WorldRef,
    ScopeRef,
    DeterminationRef,
    GrantRef,
    AuthorityRef,
    BoundsRef,
    ReturnRelationRef,
    ReturnRef,
    ActuationRef,
    RealisedRef,
    ActivityRef,
    StreamRef,
    EventRef,
    AgentSessionRef,
    RunRef,
    JourneyRef,
    PlanRef,
    InvocationRef,
    RequestRef,
    ActionRef,
    LocusRef,
    ExternalRef
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgencyIdentity {
    pub agent: AgentRef,
    pub agency: AgencyRef,
}
impl AgencyIdentity {
    /// A new body is not an input to identity. This comparison does not mint
    /// refs, infer ancestry or decide whether an explicit derivation is allowed.
    pub fn is_continuation_of(&self, other: &Self) -> bool {
        self == other
    }
}
