use actuation_core::{is_blank_reference, Error, Result};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use time::{
    format_description::well_known::{Rfc2822, Rfc3339},
    Date, OffsetDateTime,
};

pub type JsonObject = Map<String, Value>;
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// The public wire contract uses non-negative ECMAScript safe integers, not
/// arbitrary u64 or floating point approximations. JSON 1 and 1.0 agree.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Count(u64);
impl Count {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub fn new(value: u64) -> Result<Self> {
        if value > MAX_SAFE_INTEGER {
            Err(Error::new("count exceeds the public safe-integer range"))
        } else {
            Ok(Self(value))
        }
    }
    pub fn get(self) -> u64 {
        self.0
    }
    pub fn next(self) -> Result<Self> {
        Self::new(self.0 + 1)
    }
}
impl<'de> Deserialize<'de> for Count {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let n = Value::deserialize(d)?;
        let value = n.as_u64().or_else(|| {
            n.as_f64()
                .filter(|n| {
                    n.is_finite() && *n >= 0.0 && *n <= MAX_SAFE_INTEGER as f64 && n.fract() == 0.0
                })
                .map(|n| n as u64)
        });
        value
            .ok_or_else(|| serde::de::Error::custom("expected non-negative safe integer"))
            .and_then(|n| Self::new(n).map_err(serde::de::Error::custom))
    }
}

/// Preserve the supplied timestamp spelling; parsing is for admission and
/// temporal ordering, never a rewrite of the caller's evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Timestamp(String);
impl Timestamp {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        parse_timestamp(&value)?;
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn unix_nanos(&self) -> i128 {
        // Only admitted immutable timestamps inhabit this type.
        parse_timestamp(&self.0)
            .expect("admitted timestamp")
            .unix_timestamp_nanos()
    }
    pub fn now() -> Result<Self> {
        let now = OffsetDateTime::now_utc();
        let milliseconds = now
            .replace_nanosecond(now.millisecond() as u32 * 1_000_000)
            .map_err(|e| Error::new(e.to_string()))?;
        Self::new(
            milliseconds
                .format(&Rfc3339)
                .map_err(|e| Error::new(e.to_string()))?,
        )
    }
}
impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        Self::new(String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
fn parse_timestamp(value: &str) -> Result<OffsetDateTime> {
    if !is_blank_reference(value) {
        if let Ok(t) = OffsetDateTime::parse(value, &Rfc3339) {
            return Ok(t);
        }
        if let Ok(t) = OffsetDateTime::parse(value, &Rfc2822) {
            return Ok(t);
        }
        if let Ok(d) = Date::parse(
            value,
            time::macros::format_description!("[year]-[month]-[day]"),
        ) {
            return Ok(d.midnight().assume_utc());
        }
        // ISO year/month forms admitted by the prior Date.parse contract.
        for suffix in ["-01", "-01-01"] {
            let expanded = format!("{value}{suffix}");
            if let Ok(d) = Date::parse(
                &expanded,
                time::macros::format_description!("[year]-[month]-[day]"),
            ) {
                return Ok(d.midnight().assume_utc());
            }
        }
    }
    Err(Error::new("expected an ISO-compatible timestamp string"))
}

pub(crate) fn object(value: &Value) -> Result<&JsonObject> {
    value
        .as_object()
        .ok_or_else(|| Error::new("expected an object"))
}
pub(crate) fn text<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .as_str()
        .filter(|s| !is_blank_reference(s))
        .ok_or_else(|| Error::new(format!("{name} must be non-empty text")))
}
pub(crate) fn json<T: Serialize>(value: &T) -> Value {
    // Domain fields contain only JSON-supported scalars/records, never arbitrary
    // host types or floating point NaN/infinity.
    serde_json::to_value(value).expect("JSON domain is serializable")
}
pub(crate) fn nonnegative(value: &serde_json::Number) -> bool {
    value.as_f64().is_some_and(|n| n.is_finite() && n >= 0.0)
}
