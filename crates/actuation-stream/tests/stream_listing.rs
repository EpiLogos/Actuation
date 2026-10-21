//! The durable stream store lists its streams, or one Agent session's, folded
//! exactly as `load` folds them. Files that do not fold are named, never
//! dropped, and the events stay in `replay`.
use actuation_core::*;
use actuation_stream::*;
use serde_json::{json, Value};
use std::fs;

fn decode<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}
fn opening(stream: &str, session: &str) -> OpenStream {
    decode(
        json!({"stream_ref": stream, "actuation_ref": "act:direct", "agency_ref": "agency:direct",
        "agent_session_ref": session, "world_binding_ref": "binding:project",
        "started_at": "2026-09-10T08:00:00Z"}),
    )
}
fn usage(stream: &str, session: &str) -> UsageOccurrence {
    let mut v: Value = serde_json::from_str(include_str!("fixtures/usage-actions.json")).unwrap();
    v["stream_ref"] = json!(stream);
    v["observation"]["actuation_ref"] = json!("act:direct");
    v["observation"]["correlation"]["agency_ref"] = json!("agency:direct");
    v["observation"]["correlation"]["agent_session_ref"] = json!(session);
    decode(v)
}

#[test]
fn an_absent_store_lists_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path().join("absent")).unwrap();
    let listing = store.list(None).unwrap();
    assert_eq!(listing.schema, STREAM_LISTING_VERSION);
    assert!(listing.streams.is_empty() && listing.invalid.is_empty());
    assert!(
        !dir.path().join("absent").exists(),
        "a listing creates nothing"
    );
}

#[test]
fn every_stream_or_one_sessions_with_its_counts_and_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    store.open(&opening("stream:b", "session:one")).unwrap();
    store.open(&opening("stream:a 雪", "session:one")).unwrap();
    store.open(&opening("stream:c", "session:two")).unwrap();
    store
        .record_usage(usage("stream:b", "session:one"))
        .unwrap();
    store
        .close(
            &StreamRef::new("stream:c").unwrap(),
            TerminalState::Closed,
            Timestamp::new("2026-09-10T09:00:00Z").unwrap(),
        )
        .unwrap();

    let all = store.list(None).unwrap();
    assert!(all.invalid.is_empty(), "{:?}", all.invalid);
    let refs: Vec<&str> = all.streams.iter().map(|s| s.stream_ref.as_str()).collect();
    assert_eq!(refs, ["stream:a 雪", "stream:b", "stream:c"]);
    let b = &all.streams[1];
    assert_eq!(b.event_count, 1);
    assert_eq!(b.model_usage_count, 1);
    assert_eq!(b.agent_session_ref.as_str(), "session:one");
    let wire = serde_json::to_value(&all).unwrap();
    assert_eq!(wire["streams"][2]["lifecycle"]["state"], "closed");
    assert_eq!(wire["streams"][0]["world_binding_ref"], "binding:project");
    assert!(wire.get("agent_session_ref").is_none());

    let one = store
        .list(Some(&AgentSessionRef::new("session:one").unwrap()))
        .unwrap();
    let refs: Vec<&str> = one.streams.iter().map(|s| s.stream_ref.as_str()).collect();
    assert_eq!(refs, ["stream:a 雪", "stream:b"]);
    assert_eq!(
        serde_json::to_value(&one).unwrap()["agent_session_ref"],
        "session:one"
    );

    // The usage record the listing counts is the one replay hands over.
    let page = store
        .replay(&StreamRef::new("stream:b").unwrap(), PageRequest::default())
        .unwrap();
    let page = serde_json::to_value(page).unwrap();
    assert_eq!(
        page["events"][0]["model_usage"]["schema"],
        "actuation.model-usage/v1"
    );
}

#[test]
fn a_torn_or_misnamed_file_is_named_not_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    store.open(&opening("stream:good", "session:one")).unwrap();
    fs::write(dir.path().join("torn.jsonl"), "{not json\n").unwrap();
    let good = fs::read_to_string(store.path(&StreamRef::new("stream:good").unwrap())).unwrap();
    fs::write(dir.path().join("elsewhere.jsonl"), good).unwrap();
    fs::write(dir.path().join("notes.txt"), "not a stream").unwrap();
    let listing = store.list(None).unwrap();
    assert_eq!(listing.streams.len(), 1);
    assert_eq!(listing.invalid.len(), 2, "{:?}", listing.invalid);
    assert!(listing
        .invalid
        .iter()
        .any(|invalid| invalid.error == "durable filename and stream_ref disagree"));
}
