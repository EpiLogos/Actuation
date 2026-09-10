//! Pure codecs for the frozen usage-record boundary. They do not invoke a
//! provider, fill missing measurements, resolve models, or copy message text.
//! These are native boundary codecs; durable usage records remain owned by
//! actuation-stream. No message content is retained by these observations.
use crate::admission::object;
use actuation_core::{Error, Result};
use actuation_stream::{Count, ModelUsageObservation, Timestamp};
fn present<'a>(v: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    v.get(key).filter(|v| !v.is_null())
}
fn text<'a>(v: &'a serde_json::Value, name: &str) -> Result<&'a str> {
    crate::admission::text(v).map_err(|_| Error::new(format!("{name} must be non-empty text")))
}
use serde_json::{json, Value};

fn native_count(value: &Value) -> Result<Value> {
    let count: Count = serde_json::from_value(value.clone())?;
    Ok(json!(count))
}
fn correlation(options: &Value, harness: &str, session: String) -> Value {
    let mut correlation = json!({"harness_ref":harness,"native_session_ref":session});
    for key in [
        "activity_ref",
        "agent_ref",
        "agency_ref",
        "agent_session_ref",
        "body_ref",
        "external_refs",
    ] {
        if let Some(value) = present(options, key) {
            correlation[key] = value.clone();
        }
    }
    correlation
}
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::String(v) => !v.is_empty(),
        Value::Number(v) => v.as_f64() != Some(0.0),
        _ => true,
    }
}
fn counts(usage: &Value, keys: &[(&str, &str)]) -> Result<Value> {
    let has = keys.iter().any(|(key, _)| present(usage, key).is_some());
    let mut result = json!({"standing": if has {"normalized-from-native"} else {"not-reported"}});
    for (source, target) in keys {
        if let Some(value) = present(usage, source) {
            result[*target] = native_count(value)?;
        }
    }
    Ok(result)
}

pub fn from_claude_code_transcript(
    native: &Value,
    options: &Value,
) -> Result<ModelUsageObservation> {
    object(native)?;
    object(options)?;
    let message = &native["message"];
    object(message)?;
    if native["type"] != "assistant"
        || native["isApiErrorMessage"] == true
        || message["model"] == "<synthetic>"
    {
        return Err(Error::new("not a provider assistant usage event"));
    }
    let id = text(&message["id"], "message.id")?;
    let model = text(&message["model"], "message.model")?;
    let session = text(&native["sessionId"], "sessionId")?;
    let actuation = text(&options["actuation_ref"], "actuation_ref")?;
    let trace = text(&options["native_trace_ref"], "native_trace_ref")?;
    let timestamp = Timestamp::new(text(&native["timestamp"], "timestamp")?)?;
    let usage = present(message, "usage").unwrap_or(&Value::Null);
    if !usage.is_null() {
        object(usage)?;
    }
    let request = native.get("requestId").filter(|v| truthy(v));
    // Native request IDs are copied as the same interpolation the source
    // boundary used. They still have no authority to manufacture an Agent.
    let native_id = request.map(|r| match r {
        Value::String(s) => s.clone(),
        _ => r.to_string(),
    });
    let invocation = present(options, "invocation_ref")
        .cloned()
        .unwrap_or_else(|| {
            json!(format!(
                "invocation:claude-code:{}",
                native_id.as_deref().unwrap_or(id)
            ))
        });
    let mut observation = json!({
        "schema":"actuation.model-usage/v1","usage_ref":format!("model-usage:claude-code:{id}"),"actuation_ref":actuation,"invocation_ref":invocation,
        "correlation":correlation(options,"harness:claude-code",format!("claude-code:session:{session}")),
        "provider":{"standing":"not-reported"},"model":{"standing":"normalized-from-native","name":model},
        "tokens":counts(usage,&[("input_tokens","input"),("output_tokens","output")])?,
        "cache":counts(usage,&[("cache_read_input_tokens","read_input"),("cache_creation_input_tokens","creation_input")])?,
        "timing":{"completed_at":timestamp,"latency":{"standing":"not-reported"}},"cost":{"standing":"not-reported"},
        "outcome":{"state":if message["stop_reason"]=="max_tokens"{"partial"}else if message["stop_reason"].is_null(){"unknown"}else{"completed"},"standing":"normalized-from-native"},
        "provenance":{"reporter_ref":"harness:claude-code","native_event_ref":format!("claude-code:message:{id}"),"native_schema":"claude-code.transcript/assistant-message","observed_at":timestamp,"raw_evidence_refs":[trace]}
    });
    if let Some(tier) = usage.get("service_tier").filter(|v| truthy(v)) {
        observation["model"]["variant"] = tier.clone();
    }
    if let Some(reason) = message.get("stop_reason").filter(|v| truthy(v)) {
        observation["outcome"]["reason"] = reason.clone();
    }
    if let Some(request) = native_id {
        observation["provenance"]["native_request_ref"] =
            json!(format!("claude-code:request:{request}"));
    }
    if let Some(tools) = present(usage, "server_tool_use") {
        let tools = object(tools)?;
        let mut entries: Vec<_> = tools.iter().collect();
        // JS Object.entries puts array-index keys first. Preserve native
        // ordering for ordinary (named) usage classes rather than alphabetise.
        fn index(key: &str) -> Option<u32> {
            key.parse::<u32>()
                .ok()
                .filter(|n| *n < u32::MAX && n.to_string() == key)
        }
        entries.sort_by(|(a, _), (b, _)| match (index(a), index(b)) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });
        let classes=entries.into_iter().map(|(name,quantity)|Ok(json!({"class":format!("server_tool_use.{name}"),"quantity":native_count(quantity)?,"unit":"requests","standing":"normalized-from-native"}))).collect::<Result<Vec<_>>>()?;
        if !classes.is_empty() {
            observation["usage_classes"] = json!(classes);
        }
    }
    let mut facts = serde_json::Map::new();
    for key in ["service_tier", "speed", "inference_geo"] {
        if let Some(value) = present(usage, key) {
            facts.insert(key.into(), value.clone());
        }
    }
    if !facts.is_empty() {
        observation["provider_facts"] = Value::Object(facts);
    }
    ModelUsageObservation::try_from(observation)
}

pub fn from_codex_exec_events(native: &Value, options: &Value) -> Result<ModelUsageObservation> {
    object(options)?;
    let events = if native.is_array() {
        native
    } else {
        object(native)?;
        &native["events"]
    }
    .as_array()
    .filter(|v| !v.is_empty())
    .ok_or_else(|| Error::new("CodexExecEvents requires native events"))?;
    let starts: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "thread.started")
        .collect();
    let ends: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "turn.completed")
        .collect();
    if starts.len() != 1 || ends.len() != 1 {
        return Err(Error::new(
            "exactly one thread.started and turn.completed are required",
        ));
    }
    let thread = text(&starts[0]["thread_id"], "thread_id")?;
    let usage = &ends[0]["usage"];
    object(usage)?;
    let actuation = text(&options["actuation_ref"], "actuation_ref")?;
    let invocation = text(&options["invocation_ref"], "invocation_ref")?;
    let trace = text(&options["native_trace_ref"], "native_trace_ref")?;
    let observed = Timestamp::new(text(&options["observed_at"], "observed_at")?)?;
    ModelUsageObservation::try_from(json!({
        "schema":"actuation.model-usage/v1","usage_ref":format!("model-usage:codex:{invocation}"),"actuation_ref":actuation,"invocation_ref":invocation,
        "correlation":correlation(options,"harness:codex",format!("codex:thread:{thread}")),
        "provider":{"standing":"not-reported"},"model":{"standing":"not-reported"},
        "tokens":{"standing":"normalized-from-native","input":native_count(&usage["input_tokens"] )?,"output":native_count(&usage["output_tokens"])?},
        "cache":{"standing":"normalized-from-native","read_input":native_count(&usage["cached_input_tokens"] )?,"creation_input":native_count(&usage["cache_write_input_tokens"])?},
        "usage_classes":[{"class":"reasoning_output","quantity":native_count(&usage["reasoning_output_tokens"] )?,"unit":"tokens","standing":"normalized-from-native"}],
        "timing":{"latency":{"standing":"not-reported"}},"cost":{"standing":"not-reported"},"outcome":{"state":"completed","standing":"normalized-from-native"},
        "provenance":{"reporter_ref":"harness:codex","native_event_ref":format!("codex:event:{invocation}:turn.completed"),"native_schema":"codex.exec-jsonl/turn.completed","observed_at":observed,"raw_evidence_refs":[trace]}
    }))
}
