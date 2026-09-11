use crate::{Error, Result};
use serde_json::{Map, Value};
pub fn object<'a>(v: &'a Value, name: &str) -> Result<&'a Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| Error::new(format!("{name} must be an object")))
}
pub fn exact(v: &Value, fields: &[&str], name: &str) -> Result<()> {
    for key in object(v, name)?.keys() {
        if !fields.contains(&key.as_str()) {
            return Err(Error::new(format!("{name}.{key} is not declared")));
        }
    }
    Ok(())
}
pub fn text(v: &Value, name: &str) -> Result<String> {
    v.as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| Error::new(format!("{name} must be non-empty text")))
}
pub fn refs(v: &Value, required: bool, name: &str) -> Result<Value> {
    if v.is_null() && !required {
        return Ok(Value::Array(vec![]));
    }
    let a = v
        .as_array()
        .ok_or_else(|| Error::new(format!("{name} requires a reference array")))?;
    if required && a.is_empty() {
        return Err(Error::new(format!("{name} must not be empty")));
    }
    for x in a {
        text(x, name)?;
    }
    Ok(v.clone())
}
pub fn strings(v: &Value, name: &str) -> Result<Value> {
    if v.is_null() {
        return Ok(Value::Array(vec![]));
    }
    if !v.as_array().is_some_and(|a| a.iter().all(Value::is_string)) {
        return Err(Error::new(format!("{name} requires strings")));
    }
    Ok(v.clone())
}
pub fn one_of(v: &Value, options: &[&str], name: &str) -> Result<String> {
    let s = text(v, name)?;
    if !options.contains(&s.as_str()) {
        return Err(Error::new(format!("invalid {name}")));
    }
    Ok(s)
}
pub fn optional_text(input: &Value, output: &mut Value, key: &str) -> Result<()> {
    if !input[key].is_null() {
        output[key] = Value::String(text(&input[key], key)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn exact_rejects_undeclared_keys() {
        let v = json!({"a": 1});
        assert!(exact(&v, &["a"], "x").is_ok());
        let v = json!({"a": 1, "b": 2});
        assert!(exact(&v, &["a"], "x").is_err());
        assert!(exact(&json!([]), &["a"], "x").is_err());
    }

    #[test]
    fn text_requires_nonempty_text() {
        assert_eq!(text(&json!(" value "), "x").unwrap(), " value ");
        assert!(text(&json!(""), "x").is_err());
        assert!(text(&json!("   "), "x").is_err());
        assert!(text(&json!(3), "x").is_err());
        assert!(text(&Value::Null, "x").is_err());
    }

    #[test]
    fn refs_distinguish_required_and_optional() {
        assert_eq!(refs(&Value::Null, false, "x").unwrap(), json!([]));
        assert!(refs(&Value::Null, true, "x").is_err());
        assert!(refs(&json!([]), true, "x").is_err());
        assert_eq!(refs(&json!([]), false, "x").unwrap(), json!([]));
        assert_eq!(refs(&json!(["a"]), true, "x").unwrap(), json!(["a"]));
        assert!(refs(&json!(["a", 1]), false, "x").is_err());
        assert!(refs(&json!("a"), false, "x").is_err());
    }

    #[test]
    fn strings_accepts_null_or_all_string_arrays() {
        assert_eq!(strings(&Value::Null, "x").unwrap(), json!([]));
        assert_eq!(strings(&json!(["a"]), "x").unwrap(), json!(["a"]));
        assert!(strings(&json!(["a", 2]), "x").is_err());
        assert!(strings(&json!("a"), "x").is_err());
    }

    #[test]
    fn one_of_admits_only_listed_options() {
        assert_eq!(one_of(&json!("b"), &["a", "b"], "x").unwrap(), "b");
        assert!(one_of(&json!("c"), &["a", "b"], "x").is_err());
        assert!(one_of(&json!(1), &["a"], "x").is_err());
    }

    #[test]
    fn optional_text_copies_only_present_values() {
        let mut out = json!({});
        optional_text(&json!({"k": "v"}), &mut out, "k").unwrap();
        assert_eq!(out["k"], json!("v"));
        optional_text(&json!({"k": null}), &mut out, "k").unwrap();
        assert_eq!(out["k"], json!("v"));
        assert!(optional_text(&json!({"k": ""}), &mut out, "k").is_err());
    }
}
