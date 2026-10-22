//! The durable store hands over the model-usage records its streams keep, or
//! one Agent session's, with what they add up to. Only an available standing
//! is summed: a record that reports no tokens or no cost is counted as not
//! reported, never as zero, and costs stay apart by currency.
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
        "agent_session_ref": session, "started_at": "2026-09-10T08:00:00Z"}),
    )
}
/// The fixture's real-shape record, placed in `stream` for `session`, with
/// its own usage and event refs and the tokens and cost given.
fn usage(stream: &str, session: &str, n: u32, tokens: Value, cost: Value) -> UsageOccurrence {
    let mut v: Value = serde_json::from_str(include_str!("fixtures/usage-actions.json")).unwrap();
    v["stream_ref"] = json!(stream);
    v["event_ref"] = json!(format!("event:usage:{stream}:{n}"));
    v["observation"]["usage_ref"] = json!(format!("model-usage:{stream}:{n}"));
    v["observation"]["actuation_ref"] = json!("act:direct");
    v["observation"]["correlation"]["agency_ref"] = json!("agency:direct");
    v["observation"]["correlation"]["agent_session_ref"] = json!(session);
    v["observation"]["tokens"] = tokens;
    v["observation"]["cost"] = cost;
    decode(v)
}
fn counted(input: u64, output: u64) -> Value {
    json!({"standing": "normalized-from-native", "input": input, "output": output})
}
fn not_reported() -> Value {
    json!({"standing": "not-reported"})
}

#[test]
fn an_absent_store_holds_no_records() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path().join("absent")).unwrap();
    let listing = store.list_usage(None).unwrap();
    assert_eq!(listing.schema, USAGE_LISTING_VERSION);
    assert!(listing.records.is_empty() && listing.invalid.is_empty());
    assert_eq!(listing.totals, UsageTotals::default());
    assert!(
        !dir.path().join("absent").exists(),
        "a listing creates nothing"
    );
}

#[test]
fn a_sessions_records_in_order_with_what_they_add_up_to() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    store.open(&opening("stream:b", "session:one")).unwrap();
    store.open(&opening("stream:a", "session:one")).unwrap();
    store.open(&opening("stream:c", "session:two")).unwrap();
    let reported = json!({"standing": "provider-reported", "amount": 0.25, "currency": "USD"});
    let derived = json!({"standing": "derived", "amount": 0.5, "currency": "USD",
        "pricing_basis": {"source_ref": "pricing:provider-page", "revision": "2026-09",
        "effective_at": "2026-09-01T00:00:00Z"}});
    let euro = json!({"standing": "provider-reported", "amount": 2, "currency": "EUR"});
    for occurrence in [
        usage("stream:b", "session:one", 1, counted(11, 8), reported),
        usage("stream:b", "session:one", 2, not_reported(), derived),
        usage(
            "stream:a",
            "session:one",
            1,
            counted(100, 40),
            not_reported(),
        ),
        usage(
            "stream:a",
            "session:one",
            2,
            json!({"standing": "normalized-from-native", "input": 7}),
            euro,
        ),
        usage(
            "stream:c",
            "session:two",
            1,
            counted(1000, 1000),
            not_reported(),
        ),
    ] {
        store.record_usage(occurrence).unwrap();
    }

    let one = store
        .list_usage(Some(&AgentSessionRef::new("session:one").unwrap()))
        .unwrap();
    assert!(one.invalid.is_empty(), "{:?}", one.invalid);
    let placed: Vec<(&str, u64)> = one
        .records
        .iter()
        .map(|r| (r.stream_ref.as_str(), r.sequence.get()))
        .collect();
    assert_eq!(
        placed,
        [
            ("stream:a", 1),
            ("stream:a", 2),
            ("stream:b", 1),
            ("stream:b", 2)
        ]
    );
    let t = &one.totals;
    assert_eq!(t.records, 4);
    // 100 + 7 + 11 from the three records that report input; the record
    // that reports none is not read as zero.
    assert_eq!(
        t.tokens_input,
        CountTotal {
            sum: 118,
            records: 3
        }
    );
    assert_eq!(
        t.tokens_output,
        CountTotal {
            sum: 48,
            records: 2
        }
    );
    assert_eq!(t.tokens_not_reported, 1);
    assert_eq!(t.cost_not_reported, 1);
    let wire = serde_json::to_value(&one).unwrap();
    assert_eq!(wire["schema"], "actuation.usage-listing/v1");
    assert_eq!(wire["agent_session_ref"], "session:one");
    assert_eq!(
        wire["totals"]["cost"],
        json!([
            {"currency": "EUR", "amount": 2.0, "records": 1, "standings": ["provider-reported"]},
            {"currency": "USD", "amount": 0.75, "records": 2, "standings": ["provider-reported", "derived"]}
        ])
    );
    // Each record is the one `actuation usage` reads, as the stream keeps it.
    let page = store
        .replay(&StreamRef::new("stream:b").unwrap(), PageRequest::default())
        .unwrap();
    let page = serde_json::to_value(page).unwrap();
    assert_eq!(
        wire["records"][2]["usage"],
        page["events"][0]["model_usage"]
    );
    assert_eq!(
        wire["records"][2]["event_ref"],
        page["events"][0]["event_ref"]
    );
    ModelUsageObservation::try_from(wire["records"][2]["usage"].clone()).unwrap();

    let all = store.list_usage(None).unwrap();
    assert_eq!(all.records.len(), 5);
    assert_eq!(
        all.totals.tokens_input,
        CountTotal {
            sum: 1118,
            records: 4
        }
    );
    assert!(serde_json::to_value(&all)
        .unwrap()
        .get("agent_session_ref")
        .is_none());
}

#[test]
fn a_torn_stream_is_named_and_its_records_are_not_guessed() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    store.open(&opening("stream:good", "session:one")).unwrap();
    store
        .record_usage(usage(
            "stream:good",
            "session:one",
            1,
            counted(3, 4),
            not_reported(),
        ))
        .unwrap();
    fs::write(dir.path().join("torn.jsonl"), "{not json\n").unwrap();
    let listing = store.list_usage(None).unwrap();
    assert_eq!(listing.records.len(), 1);
    assert_eq!(listing.invalid.len(), 1, "{:?}", listing.invalid);
    assert_eq!(
        listing.totals.tokens_output,
        CountTotal { sum: 4, records: 1 }
    );
}
