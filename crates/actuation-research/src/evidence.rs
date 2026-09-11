use crate::{value::*, Error, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
/// Cross-language digest of semantic JSON. Object ordering follows the frozen
/// JavaScript oracle (UTF-16 key sort; integer-index enumeration before text).
pub fn stable_digest(value: &Value) -> String {
    fn number(n: &serde_json::Number) -> String {
        let f = n.as_f64().expect("JSON number");
        if f == 0.0 {
            return "0".into();
        }
        let s = f.to_string();
        let (sign, body) = s.strip_prefix('-').map_or(("", s.as_str()), |s| ("-", s));
        if (1e-6..1e21).contains(&f.abs()) {
            return s;
        }
        let (base, explicit) = body
            .split_once('e')
            .map_or((body, 0), |(b, e)| (b, e.parse::<i32>().unwrap_or(0)));
        let point = base.find('.').unwrap_or(base.len()) as i32;
        let raw = base.replace('.', "");
        let leading = raw.bytes().take_while(|b| *b == b'0').count();
        let digits = raw[leading..].trim_end_matches('0');
        let exponent = point - leading as i32 - 1 + explicit;
        let mantissa = if digits.len() == 1 {
            digits.to_owned()
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        format!(
            "{sign}{mantissa}e{}{exponent}",
            if exponent >= 0 { "+" } else { "" }
        )
    }
    fn render(v: &Value) -> String {
        match v {
            Value::Object(m) => {
                let mut names = m.keys().collect::<Vec<_>>();
                let index = |s: &str| {
                    s.parse::<u32>()
                        .ok()
                        .filter(|n| *n < u32::MAX && n.to_string() == s)
                };
                names.sort_by(|a, b| match (index(a), index(b)) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    _ => a.encode_utf16().cmp(b.encode_utf16()),
                });
                format!(
                    "{{{}}}",
                    names
                        .into_iter()
                        .map(|k| format!("{}:{}", serde_json::to_string(k).unwrap(), render(&m[k])))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
            Value::Array(a) => format!("[{}]", a.iter().map(render).collect::<Vec<_>>().join(",")),
            Value::Number(n) => number(n),
            _ => v.to_string(),
        }
    }
    format!("{:x}", Sha256::digest(render(value)))
}
pub fn bytes_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
/// These are observed native handles, NOT admitted Agent/Agency identities.
/// Raw records are retained by the runner so a derived family remains auditable.
pub fn extract_prime_family(records: &[Value]) -> Value {
    fn coalesce<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
        keys.iter()
            .find_map(|k| v.get(*k).filter(|v| !v.is_null()))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
    }
    fn upsert(nodes: &mut Map<String, Value>, id: &str, extra: Value) {
        if id.is_empty() {
            return;
        }
        let n = nodes
            .entry(id.to_owned())
            .or_insert_with(|| json!({"id":id}));
        n.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        if n["rlm_child_id"].is_null() && id.starts_with("sub-") {
            n["rlm_child_id"] = json!(id);
        }
    }
    fn visit(
        v: &Value,
        nodes: &mut Map<String, Value>,
        edges: &mut Vec<Value>,
        patterns: &[Regex; 3],
    ) {
        let mut edge = |parent: &str, child: &str| {
            let e = json!({"parent":parent,"child":child});
            if !edges.contains(&e) {
                edges.push(e)
            }
        };
        match v {
            Value::Object(m) => {
                let child = coalesce(
                    v,
                    &[
                        "rlm_child_id",
                        "rlmChildId",
                        "child_id",
                        "childId",
                        "child_ref",
                    ],
                );
                let active = coalesce(
                    v,
                    &[
                        "active_session_id",
                        "activeSessionId",
                        "session_id",
                        "sessionId",
                    ],
                );
                let parent = coalesce(
                    v,
                    &[
                        "parent_session_id",
                        "parentSessionId",
                        "parent_id",
                        "parentId",
                        "parent_ref",
                    ],
                );
                if let Some(id) = child.or(active) {
                    let mut extras = json!({});
                    if let Some(name) = coalesce(v, &["session_name", "sessionName", "name"]) {
                        extras["name"] = json!(name)
                    }
                    if let Some(active) = active {
                        extras["active_session_id"] = json!(active)
                    }
                    if let Some(child) = child {
                        extras["rlm_child_id"] = json!(child)
                    }
                    upsert(nodes, id, extras);
                    if let Some(p) = parent.filter(|p| *p != id) {
                        edge(p, id);
                    }
                }
                for v in m.values() {
                    visit(v, nodes, edges, patterns);
                }
            }
            Value::Array(a) => {
                for v in a {
                    visit(v, nodes, edges, patterns)
                }
            }
            Value::String(s) => {
                for p in &patterns[..2] {
                    for c in p.captures_iter(s) {
                        upsert(nodes, &c[1], json!({"rlm_child_id":&c[1]}));
                    }
                }
                for c in patterns[2].captures_iter(s) {
                    upsert(nodes, &c[2], json!({"rlm_child_id":&c[2]}));
                    edge(&c[1], &c[2]);
                }
            }
            _ => {}
        }
    }
    let patterns = [
        Regex::new(r#"rlm_child_id["']?\s*[:=]\s*["']?(sub-[A-Za-z0-9_-]+)"#).unwrap(),
        Regex::new(r#""child_ref"\s*:\s*"([^"]+)""#).unwrap(),
        Regex::new(r#""parent_ref"\s*:\s*"([^"]+)"[\s\S]{0,500}?"child_ref"\s*:\s*"([^"]+)""#)
            .unwrap(),
    ];
    let mut nodes = Map::new();
    let mut edges = Vec::new();
    for v in records {
        visit(v, &mut nodes, &mut edges, &patterns);
    }
    let children = nodes
        .values()
        .filter(|v| v["rlm_child_id"].is_string())
        .cloned()
        .collect::<Vec<_>>();
    let child_ids = children
        .iter()
        .filter_map(|v| v["id"].as_str())
        .collect::<Vec<_>>();
    let nested = edges
        .iter()
        .filter(|v| {
            child_ids.contains(&v["parent"].as_str().unwrap_or(""))
                && child_ids.contains(&v["child"].as_str().unwrap_or(""))
        })
        .cloned()
        .collect::<Vec<_>>();
    json!({"nodes":nodes.into_values().collect::<Vec<_>>(),"edges":edges,"child_nodes":children,"nested_edges":nested})
}
pub fn candidate_boundary(v: &Value) -> Result<()> {
    let denied = [
        "reviewreference",
        "review_reference",
        "human_reference",
        "includeforhumanreview",
        "expectedanswer",
        "expected_answer",
    ];
    match v {
        Value::Array(a) => {
            for v in a {
                candidate_boundary(v)?
            }
        }
        Value::Object(m) => {
            for (k, v) in m {
                if denied.contains(&k.to_lowercase().as_str()) {
                    return Err(Error::new(format!(
                        "human-review-only field crossed candidate boundary: {k}"
                    )));
                }
                candidate_boundary(v)?
            }
        }
        _ => {}
    }
    Ok(())
}
pub fn sanitize(v: &Value, secrets: &[String]) -> Value {
    let replace = |s: &str| {
        secrets
            .iter()
            .filter(|s| !s.is_empty())
            .fold(s.to_owned(), |v, s| v.replace(s, "[REDACTED]"))
    };
    match v {
        Value::String(s) => json!(replace(s)),
        Value::Array(a) => json!(a.iter().map(|v| sanitize(v, secrets)).collect::<Vec<_>>()),
        Value::Object(m) => {
            let secret=Regex::new("(?i)(api[_-]?key|authorization|credential|password|secret|access[_-]?token|refresh[_-]?token|bearer)").unwrap();
            Value::Object(
                m.iter()
                    .map(|(k, v)| {
                        (
                            replace(k),
                            if k != "credential_contract" && secret.is_match(k) {
                                json!("[REDACTED]")
                            } else {
                                sanitize(v, secrets)
                            },
                        )
                    })
                    .collect(),
            )
        }
        _ => v.clone(),
    }
}
pub fn assert_no_secrets(v: &Value, secrets: &[String]) -> Result<()> {
    let serialized = v.to_string();
    if secrets
        .iter()
        .any(|s| !s.is_empty() && serialized.contains(s))
    {
        Err(Error::new("evidence contains configured secret"))
    } else {
        Ok(())
    }
}
pub fn prime_return(input: &Value, require_schema: bool) -> Result<Value> {
    object(input, "Prime Return")?;
    if require_schema && input["schema"] != "actuation.prime-return/v0" {
        return Err(Error::new("wrong Prime Return schema"));
    }
    let mut r = json!({"schema":"actuation.prime-return/v0"});
    for k in [
        "subject_ref",
        "relation_to_parent",
        "determination",
        "result",
        "difference",
    ] {
        r[k] = json!(text(&input[k], k)?);
    }
    for k in [
        "evidence_refs",
        "ql_reading_refs",
        "unresolved",
        "next_relations",
    ] {
        r[k] = strings(&input[k], k)?;
    }
    r["provenance"] = if input["provenance"].is_object() {
        input["provenance"].clone()
    } else {
        json!({})
    };
    for k in ["child_ref", "parent_ref"] {
        optional_text(input, &mut r, k)?;
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_digest_matches_known_sha256_vector() {
        assert_eq!(
            bytes_digest(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            bytes_digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn stable_digest_is_order_independent_over_objects_only() {
        let a = stable_digest(&json!({"a":1,"b":2}));
        let b = stable_digest(&json!({"b":2,"a":1}));
        assert_eq!(a, b, "object key order must not change the digest");
        assert_ne!(
            stable_digest(&json!([1, 2])),
            stable_digest(&json!([2, 1])),
            "array order is semantic"
        );
        assert_ne!(a, stable_digest(&json!({"a":2,"b":1})));
    }

    #[test]
    fn stable_digest_normalises_numbers_like_the_frozen_oracle() {
        assert_eq!(
            stable_digest(&json!({"x":0})),
            stable_digest(&json!({"x":0.0})),
            "integer and float zero render identically"
        );
        assert_ne!(
            stable_digest(&json!({"x":0})),
            stable_digest(&json!({"x":-1})),
        );
        // Integer-named keys enumerate before text keys (UTF-16 order otherwise).
        assert_eq!(
            stable_digest(&json!({"2":"b","10":"c","a":"z"})),
            stable_digest(&json!({"a":"z","2":"b","10":"c"})),
        );
    }

    #[test]
    fn candidate_boundary_blocks_review_only_fields_at_any_depth() {
        assert!(candidate_boundary(&json!({"prompt":"p","successConditions":["s"]})).is_ok());
        assert!(candidate_boundary(&json!({"nested":{"review_reference":"x"}})).is_err());
        assert!(candidate_boundary(&json!({"expectedAnswer":"x"})).is_err());
        assert!(candidate_boundary(&json!([{"human_reference":1}])).is_err());
        assert!(candidate_boundary(&json!("plain string")).is_ok());
    }

    #[test]
    fn sanitize_redacts_key_names_and_configured_values() {
        let v = json!({"apiKey":"k-123","credential_contract":"keep:secretvalue","note":"token secretvalue here","list":["secretvalue"]});
        let out = sanitize(&v, &["secretvalue".to_owned()]);
        assert_eq!(out["apiKey"], json!("[REDACTED]"));
        // The key survives; its string value is still value-redacted.
        assert_eq!(out["credential_contract"], json!("keep:[REDACTED]"));
        assert_eq!(out["note"], json!("token [REDACTED] here"));
        assert_eq!(out["list"], json!(["[REDACTED]"]));
        assert!(assert_no_secrets(&out, &["secretvalue".to_owned()]).is_ok());
        assert!(assert_no_secrets(&v, &["secretvalue".to_owned()]).is_err());
    }

    #[test]
    fn prime_return_validates_its_envelope() {
        let valid = json!({
            "schema":"actuation.prime-return/v0",
            "subject_ref":"circuit:1",
            "relation_to_parent":"bounded local whole of the parent evaluation",
            "determination":"confirmed the parent reading",
            "result":"the sum contract holds after the fix",
            "difference":"child found an extra edge case the parent missed",
            "evidence_refs":["evidence:1"],
            "ql_reading_refs":[],
            "unresolved":["unresolved:1"],
            "next_relations":[],
            "provenance":{"scenario":"test"}
        });
        let r = prime_return(&valid, true).expect("valid return");
        assert_eq!(r["child_ref"], Value::Null);
        let mut missing = valid.clone();
        missing["difference"] = json!("");
        assert!(prime_return(&missing, true).is_err());
        let mut wrong = valid.clone();
        wrong["schema"] = json!("other/v0");
        assert!(prime_return(&wrong, true).is_err());
        assert!(prime_return(&wrong, false).is_ok());
    }

    #[test]
    fn extract_prime_family_keeps_observed_handles_only() {
        let records = vec![
            json!({"session_id":"root-session","rlm_child_id":"sub-1"}),
            json!({"type":"log","line":"spawned rlm_child_id: \"sub-2\" for material work"}),
            json!({"parent_ref":"root-session","child_ref":"sub-2"}),
            json!({"session_id":"observer-only"}),
        ];
        let family = extract_prime_family(&records);
        let ids: Vec<&str> = family["child_nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["id"].as_str().unwrap())
            .collect();
        assert!(ids.contains(&"sub-1") && ids.contains(&"sub-2"));
        assert!(family["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["id"] == json!("sub-1") && n["active_session_id"] == json!("root-session")));
        assert!(family["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["id"] == json!("observer-only") && n["rlm_child_id"].is_null()));
        assert_eq!(
            family["edges"],
            json!([{"parent":"root-session","child":"sub-2"}])
        );
        // A nested edge requires the parent itself to be an observed child locus.
        assert_eq!(family["nested_edges"], json!([]));
    }
}
