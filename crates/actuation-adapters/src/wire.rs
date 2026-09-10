use actuation_core::{is_blank_reference, Error, Result};
use actuation_stream::Timestamp;
use serde_json::{Map, Value};

pub fn object(v: &Value) -> Result<&Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| Error::new("expected an object"))
}
pub fn text<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    v.as_str()
        .filter(|s| !is_blank_reference(s))
        .ok_or_else(|| Error::new(format!("{name} must be a non-empty string")))
}
pub fn present<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.get(key).filter(|v| !v.is_null())
}
pub fn optional_text(v: &Value, key: &str) -> Result<()> {
    if let Some(value) = present(v, key) {
        text(value, key)?;
    }
    Ok(())
}
pub fn list<'a>(v: &'a Value, name: &str) -> Result<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| Error::new(format!("{name} must be an array")))
}
pub fn strings<'a>(v: &'a Value, name: &str) -> Result<Vec<&'a str>> {
    list(v, name)?.iter().map(|v| text(v, name)).collect()
}
pub fn optional_strings(v: &Value, key: &str) -> Result<()> {
    if let Some(value) = present(v, key) {
        strings(value, key)?;
    }
    Ok(())
}
pub fn choice<'a>(v: &'a Value, choices: &[&str], name: &str) -> Result<&'a str> {
    let s = text(v, name)?;
    if !choices.contains(&s) {
        return Err(Error::new(format!(
            "{name} must be one of {}",
            choices.join(", ")
        )));
    }
    Ok(s)
}
pub fn exact(v: &Value, expected: &str, name: &str) -> Result<()> {
    if v != expected {
        return Err(Error::new(format!("{name} must equal {expected}")));
    }
    Ok(())
}
pub fn boolean(v: &Value, name: &str) -> Result<bool> {
    v.as_bool()
        .ok_or_else(|| Error::new(format!("{name} must be boolean")))
}
pub fn integer(v: &Value, name: &str) -> Result<()> {
    if v.as_f64()
        .is_none_or(|n| !n.is_finite() || n.fract() != 0.0)
    {
        return Err(Error::new(format!("{name} must be an integer")));
    }
    Ok(())
}
pub fn timestamp(v: &Value, name: &str) -> Result<()> {
    Timestamp::new(text(v, name)?)?;
    Ok(())
}
pub fn fingerprint(v: &Value) -> Result<()> {
    if !v.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err(Error::new("expected a lowercase SHA-256 fingerprint"));
    }
    Ok(())
}
pub fn facts(v: &Value) -> Result<()> {
    if v.is_null() {
        return Ok(());
    }
    for (key, value) in object(v)? {
        if is_blank_reference(key)
            || !(value.is_null() || value.is_boolean() || value.is_number() || value.is_string())
        {
            return Err(Error::new("native facts must be named scalar observations"));
        }
    }
    Ok(())
}
pub fn required_texts(v: &Value, fields: &[&str]) -> Result<()> {
    for key in fields {
        text(&v[*key], key)?;
    }
    Ok(())
}
pub fn optional_texts(v: &Value, fields: &[&str]) -> Result<()> {
    for key in fields {
        optional_text(v, key)?;
    }
    Ok(())
}
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::String(v) => !v.is_empty(),
        Value::Number(v) => v.as_f64() != Some(0.0),
        _ => true,
    }
}
pub fn owned_strings(v: &Value, name: &str) -> Result<Vec<String>> {
    Ok(strings(v, name)?.into_iter().map(str::to_owned).collect())
}

// Wire documents retain published extensions and omission/null spelling. They
// are immutable after admission; actual observation/construction is implemented
// by the domain's methods and ports, not by the test transport.
macro_rules! document {
    ($name:ident, $check:path) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
        #[serde(transparent)]
        pub struct $name(serde_json::Value);
        impl $name {
            pub fn new(value: serde_json::Value) -> actuation_core::Result<Self> {
                $check(&value)?;
                Ok(Self(value))
            }
            pub fn as_value(&self) -> &serde_json::Value {
                &self.0
            }
            pub fn into_value(self) -> serde_json::Value {
                self.0
            }
        }
        impl TryFrom<serde_json::Value> for $name {
            type Error = actuation_core::Error;
            fn try_from(value: serde_json::Value) -> actuation_core::Result<Self> {
                Self::new(value)
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(
                d: D,
            ) -> std::result::Result<Self, D::Error> {
                Self::new(serde_json::Value::deserialize(d)?).map_err(serde::de::Error::custom)
            }
        }
    };
}
pub(crate) use document;
