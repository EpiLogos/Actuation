#![allow(dead_code)]
// Test transport only. Production depends on typed library APIs, never this
// dispatcher, fixture file, captured answers, or an embedded JavaScript host.
use actuation_core::*;
use actuation_stream::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};

pub fn encode<T: Serialize>(value: T) -> Result<Value> {
    Ok(serde_json::to_value(value)?)
}
fn admitted<T: DeserializeOwned + Serialize>(value: Value) -> Result<Value> {
    encode(serde_json::from_value::<T>(value)?)
}
fn obj(v: &Value) -> Result<()> {
    if v.is_object() {
        Ok(())
    } else {
        Err(Error::new("options must be an object"))
    }
}
fn arg(args: &[Value], index: usize) -> Value {
    args.get(index).cloned().unwrap_or(Value::Null)
}
fn options(args: &[Value], index: usize) -> Result<Value> {
    let v = args.get(index).cloned().unwrap_or(json!({}));
    obj(&v)?;
    Ok(v)
}
pub fn page(options: &Value, durable: bool) -> Result<PageRequest> {
    obj(options)?;
    let after = serde_json::from_value(options.get("afterSequence").cloned().unwrap_or(json!(0)))?;
    let limit = match options.get("limit") {
        None => None,
        Some(Value::Null) if durable => None,
        Some(v) => Some(serde_json::from_value(v.clone())?),
    };
    Ok(PageRequest {
        after_sequence: after,
        limit,
    })
}
pub fn evaluate(operation: &str, args: &[Value]) -> Result<Value> {
    let first = arg(args, 0);
    match operation {
        "contracts/actuation-stream.mjs#validateActuationStream" => {
            admitted::<ActuationStream>(first)
        }
        "contracts/actuation-stream.mjs#validateActuationStreamEvent" => {
            let event = StreamEvent::try_from(first)?;
            let o = options(args, 1)?;
            if let Some(sequence) = o.get("expectedSequence").filter(|v| !v.is_null()) {
                event.assert_sequence(serde_json::from_value(sequence.clone())?)?;
            }
            encode(event)
        }
        "contracts/actuation-stream.mjs#actuationStreamReadModel" => {
            encode(ActuationStream::try_from(first)?.read(page(&options(args, 1)?, false)?))
        }
        "contracts/actuation-stream.mjs#appendActuationStreamEvent" => {
            encode(ActuationStream::try_from(first)?.append(StreamEvent::try_from(arg(args, 1))?)?)
        }
        "contracts/actuation-stream.mjs#closeActuationStream" => {
            let o = options(args, 1)?;
            let state = serde_json::from_value(o.get("state").cloned().unwrap_or(json!("closed")))?;
            encode(
                ActuationStream::try_from(first)?
                    .close(state, serde_json::from_value(o["endedAt"].clone())?)?,
            )
        }
        "contracts/activity.mjs#validateActivity" => admitted::<Activity>(first),
        "contracts/activity.mjs#activityNeedsAttention" => {
            Ok(json!(Activity::try_from(first)?.needs_attention()))
        }
        "contracts/activity.mjs#activityFromActuationStream" => encode(Activity::from_stream(
            &ActuationStream::try_from(first)?,
            serde_json::from_value(arg(args, 1))?,
        )?),
        "contracts/activity.mjs#activityFromStreamEvent" => {
            let mut o = options(args, 2)?;
            let phase =
                serde_json::from_value(o.get("phase").cloned().unwrap_or(json!("completed")))?;
            o.as_object_mut().unwrap().remove("metadata");
            encode(Activity::from_event(
                &ActuationStream::try_from(first)?,
                &serde_json::from_value(arg(args, 1))?,
                serde_json::from_value(o)?,
                phase,
            )?)
        }
        "contracts/model-usage.mjs#validateModelUsageObservation" => {
            admitted::<ModelUsageObservation>(first)
        }
        "contracts/request-correlation.mjs#validateAuthorityDecision"
        | "contracts/request-correlation.mjs#authorityDecision" => {
            admitted::<AuthorityDecision>(first)
        }
        "contracts/request-correlation.mjs#requestCorrelationReadModel" => {
            Ok(CorrelationCorpus::try_from(options(args, 1)?)?
                .correlate(&serde_json::from_value(first)?))
        }
        _ => Err(Error::new(format!(
            "unsupported stream oracle operation {operation}"
        ))),
    }
}
pub fn caught(value: Result<Value>) -> Value {
    match value {
        Ok(value) => json!({"ok":true,"value":value}),
        Err(error) => json!({"ok":false,"error":{"name":"TypeError","message":error.to_string()}}),
    }
}
pub fn pure_row(row: &Value) -> Value {
    let result = match (row["operation"].as_str(), row["args"].as_array()) {
        (Some(operation), Some(args)) => evaluate(operation, args),
        _ => Err(Error::new("invalid oracle request")),
    };
    let mut answer = caught(result);
    answer["id"] = row["id"].clone();
    answer
}

pub fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../../fixtures/migration/scenarios.json"
    ))
    .unwrap()
}
pub struct FixtureCatalogue(Value);
impl FixtureCatalogue {
    pub fn new() -> Self {
        Self(corpus()["cases"][0]["expected"]["capabilities"].clone())
    }
}
impl Default for FixtureCatalogue {
    fn default() -> Self {
        Self::new()
    }
}
impl BoundaryCatalogue for FixtureCatalogue {
    fn boundary(&self, harness: &str, native_event: &str) -> Result<NativeBoundary> {
        let d = self
            .0
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["harness_slug"] == harness)
            .ok_or_else(|| Error::new("no capability descriptor for harness"))?;
        let event = d["native_events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["native_name"] == native_event)
            .ok_or_else(|| Error::new("undeclared native event"))?;
        Ok(NativeBoundary {
            harness: ExternalRef::new(harness)?,
            native_event: ExternalRef::new(native_event)?,
            boundary: if event["event"] == "custom" {
                None
            } else {
                Some(serde_json::from_value(event["event"].clone())?)
            },
            catalog_revision: serde_json::from_value(d["provenance"]["catalog_revision"].clone())?,
        })
    }
}
pub fn store_action(store: &JsonlStreamStore, operation: &str, args: &Value) -> Result<Value> {
    obj(args)?;
    match operation {
        "openDurableStream" => encode(
            store
                .open(&serde_json::from_value(args.clone())?)?
                .read(PageRequest::default()),
        ),
        "loadDurableStream" => {
            encode(store.load(&serde_json::from_value(args["stream_ref"].clone())?)?)
        }
        "replayDurableStream" => encode(store.replay(
            &serde_json::from_value(args["stream_ref"].clone())?,
            page(args, true)?,
        )?),
        "closeDurableStream" => {
            let end = match args.get("ended_at").filter(|v| !v.is_null()) {
                Some(v) => serde_json::from_value(v.clone())?,
                None => Timestamp::now()?,
            };
            encode(
                store
                    .close(
                        &serde_json::from_value(args["stream_ref"].clone())?,
                        serde_json::from_value(
                            args.get("state").cloned().unwrap_or(json!("closed")),
                        )?,
                        end,
                    )?
                    .read(PageRequest::default()),
            )
        }
        "recordBoundaryOccurrence" => encode(store.record_boundary(
            serde_json::from_value(args.clone())?,
            &FixtureCatalogue::new(),
        )?),
        "recordModelUsageObservation" => {
            encode(store.record_usage(serde_json::from_value(args.clone())?)?)
        }
        "append" => encode(store.append(
            &serde_json::from_value(args["stream_ref"].clone())?,
            StreamEvent::try_from(args["event"].clone())?,
        )?),
        _ => Err(Error::new("unsupported store oracle operation")),
    }
}
fn files(root: &Path) -> Result<BTreeMap<String, String>> {
    fs::read_dir(root)
        .map_err(|e| Error::new(e.to_string()))?
        .map(|entry| {
            let entry = entry.map_err(|e| Error::new(e.to_string()))?;
            Ok((
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| Error::new("non-UTF8 store filename"))?,
                fs::read_to_string(entry.path()).map_err(|e| Error::new(e.to_string()))?,
            ))
        })
        .collect()
}
pub fn scenario(row: &Value) -> Result<Value> {
    let input = &row["input"];
    match row["kind"].as_str().unwrap_or("") {
        "fold" => Ok(caught(
            fold_stream_file(
                input["raw"]
                    .as_str()
                    .ok_or_else(|| Error::new("raw must be text"))?,
            )
            .and_then(encode),
        )),
        "filename" => Ok(caught(
            serde_json::from_value::<StreamRef>(input["ref"].clone())
                .map(|r| json!(stream_file_name(&r)))
                .map_err(Into::into),
        )),
        "command" => {
            let store = JsonlStreamStore::new(
                input["root"]
                    .as_str()
                    .ok_or_else(|| Error::new("root is required for test command"))?,
            )?;
            Ok(caught(store_action(
                &store,
                input["operation"].as_str().unwrap_or(""),
                &input["args"],
            )))
        }
        "store" => {
            let root = tempfile::tempdir().map_err(|e| Error::new(e.to_string()))?;
            let store = JsonlStreamStore::new(root.path())?;
            let mut results = Vec::new();
            for action in input["actions"]
                .as_array()
                .ok_or_else(|| Error::new("missing actions"))?
            {
                let before = files(root.path())?;
                if let Some(raw) = action.get("write") {
                    let stream_ref: StreamRef =
                        serde_json::from_value(action["stream_ref"].clone())?;
                    fs::write(store.path(&stream_ref), raw.as_str().unwrap())
                        .map_err(|e| Error::new(e.to_string()))?;
                }
                let result = caught(store_action(
                    &store,
                    action["operation"].as_str().unwrap_or(""),
                    &action["args"],
                ));
                if let Some(expected) = action.get("expect_ok") {
                    if &result["ok"] != expected {
                        return Err(Error::new(format!(
                            "authored assertion failed: {}: {}",
                            action["label"], result
                        )));
                    }
                }
                let after = files(root.path())?;
                if action["unchanged"] == true && before != after {
                    return Err(Error::new("refusal/dedup wrote to the durable store"));
                }
                if action["append_only"] == true {
                    for (name, raw) in &before {
                        if !after.get(name).is_some_and(|next| next.starts_with(raw)) {
                            return Err(Error::new("append rewrote existing bytes"));
                        }
                    }
                }
                results.push(json!({"label":action["label"],"result":result,"files":after}));
            }
            Ok(json!(results))
        }
        _ => Err(Error::new("unsupported store scenario kind")),
    }
}
pub fn normalized(mut value: Value) -> Value {
    match &mut value {
        Value::Array(rows) => {
            for row in rows {
                *row = normalized(row.take());
            }
        }
        Value::Object(fields) => {
            if fields.get("ok") == Some(&json!(false)) && fields.contains_key("error") {
                return json!({"ok":false});
            }
            for (_, v) in fields.iter_mut() {
                *v = normalized(v.take());
            }
            if let Some(Value::Object(files)) = fields.get_mut("files") {
                for (_, v) in files.iter_mut() {
                    let raw = v.as_str().unwrap();
                    *v = json!(raw
                        .split('\n')
                        .map(|line| serde_json::from_str::<Value>(line).unwrap_or(json!(line)))
                        .collect::<Vec<_>>());
                }
            }
        }
        _ => {}
    }
    value
}
