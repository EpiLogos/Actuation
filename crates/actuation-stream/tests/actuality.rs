mod support;
use actuation_core::*;
use actuation_stream::*;
use serde_json::{json, Value};
use std::{
    fs,
    sync::{Arc, Barrier},
    thread,
};

fn decode<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}
fn opening() -> OpenStream {
    decode(
        json!({"stream_ref":"stream:direct/雪 !'()*","actuation_ref":"act:direct","agency_ref":"agency:direct","agent_session_ref":"session:external","world_binding_ref":"binding:project","started_at":"2026-09-10T08:00:00Z"}),
    )
}
fn event(sequence: u64) -> StreamEvent {
    decode(
        json!({"event_ref":format!("event:{sequence}"),"kind":"world-observation","sequence":sequence,"observed_at":"2026-09-10T08:00:01Z","content":"observed difference","native_trace_ref":"trace:external","actor":{"agent_ref":"agent:one","agency_ref":"agency:direct","locus_ref":"locus:ordinary"}}),
    )
}
fn description() -> ActivityDescription {
    decode(
        json!({"activityRef":"activity:direct","subjectRef":"world:project","nativeOwner":"actuation","verb":"observed","object":"native occurrence","summary":"An actual external event","actionDescription":"An external locus returned a difference","locationDescription":"The existing project World","why":"Preserve actual evidence before recognition"}),
    )
}
fn boundary(index: usize) -> BoundaryOccurrence {
    decode(
        json!({"stream_ref":opening().stream_ref,"harness":"claude-code","native_event":"PreToolUse","event_ref":format!("native-event:{index}"),"observed_at":"2026-09-10T08:00:01Z"}),
    )
}
fn usage() -> UsageOccurrence {
    let corpus = support::corpus();
    let mut v = corpus["cases"][1]["input"]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == "usage")
        .unwrap()["args"]
        .clone();
    v["stream_ref"] = json!(opening().stream_ref);
    v["observation"]["actuation_ref"] = json!(opening().actuation_ref);
    v["observation"]["correlation"]["agency_ref"] = json!(opening().agency_ref);
    v["observation"]["correlation"]["agent_session_ref"] = json!(opening().agent_session_ref);
    decode(v)
}
#[test]
fn direct_actuality_retains_its_world_actor_and_trace_without_factory_ancestry() {
    let stream = opening().empty().unwrap().append(event(1)).unwrap();
    let activity = Activity::from_stream(&stream, description()).unwrap();
    let v = serde_json::to_value(activity).unwrap();
    assert_eq!(v["actor"]["agent_ref"], "agent:one");
    assert_eq!(v["actor"]["agency_ref"], "agency:direct");
    assert_eq!(v["actuation_ref"], "act:direct");
    for key in [
        "run_ref",
        "plan_ref",
        "journey_ref",
        "action_ref",
        "recognition_ref",
    ] {
        assert!(v.get(key).is_none_or(Value::is_null), "fabricated {key}");
    }
    assert_eq!(v["trace"]["stream_ref"], json!(opening().stream_ref));
    assert_eq!(v["needs_attention"], false);
    assert_eq!(
        stream.fields().world_binding_ref.value(),
        Some(&opening().world_binding_ref.unwrap())
    );
}
#[test]
fn non_action_event_is_activity_and_factory_refs_remain_supplied_correlations() {
    let stream = opening().empty().unwrap().append(event(1)).unwrap();
    let mut description=serde_json::to_value(json!({"activityRef":"activity:one","subjectRef":"world:project","nativeOwner":"actuation","verb":"observed","object":"native occurrence","summary":"An actual external event","actionDescription":"Observed native occurrence","locationDescription":"Project","planRef":"plan:owner","journeyRef":"journey:owner","runRef":"run:owner","salience":"critical","needsAttention":true})).unwrap();
    let activity = Activity::from_event(
        &stream,
        &event(1).fields().event_ref,
        decode(description.take()),
        ActivityPhase::Completed,
    )
    .unwrap();
    let v = serde_json::to_value(activity).unwrap();
    assert_eq!(v["run_ref"], "run:owner");
    assert_eq!(v["journey_ref"], "journey:owner");
    assert_eq!(v["actuation_ref"], "act:direct");
    assert_ne!(v["run_ref"], v["actuation_ref"]);
    assert!(v["action_ref"].is_null());
    assert_eq!(v["needs_attention"], true);
    assert_eq!(v["completed_at"], "2026-09-10T08:00:01Z");
}
#[test]
fn only_explicit_actor_facts_are_admitted_and_event_role_rules_do_not_invent_an_agent() {
    assert!(serde_json::from_value::<Attribution>(json!({})).is_err());
    assert!(serde_json::from_value::<StreamEvent>(json!({"event_ref":"event:x","kind":"world-observation","sequence":1,"actor":{"agent_ref":"agent:one","role":""}})).is_err());
    let e: StreamEvent = decode(
        json!({"event_ref":"event:x","kind":"world-observation","sequence":1,"observed_at":"2026-09-10T08:00:01Z"}),
    );
    let activity = Activity::from_event(
        &opening().empty().unwrap().append(e.clone()).unwrap(),
        &e.fields().event_ref,
        description(),
        ActivityPhase::Completed,
    )
    .unwrap();
    let v = serde_json::to_value(activity).unwrap();
    assert_eq!(v["actor"]["agency_ref"], "agency:direct");
    assert!(v["actor"]["agent_ref"].is_null());
}
#[test]
fn missing_clock_is_not_reconstructed_as_an_observation() {
    let mut o = opening();
    o.started_at = None;
    assert!(Activity::from_stream(&o.empty().unwrap(), description()).is_err());
    let e: StreamEvent =
        decode(json!({"event_ref":"event:x","kind":"world-observation","sequence":1}));
    assert!(Activity::from_event(
        &o.empty().unwrap().append(e.clone()).unwrap(),
        &e.fields().event_ref,
        description(),
        ActivityPhase::Completed
    )
    .is_err());
}
#[test]
fn omissions_nulls_and_open_extensions_round_trip_without_shadowing_identity() {
    let base = serde_json::to_value(opening().empty().unwrap()).unwrap();
    for extension in [Value::Null, json!({"foreign":[1,"actual",null]})] {
        let mut v = base.clone();
        v["future_owner_field"] = extension;
        v["surface_refs"] = Value::Null;
        let stream = ActuationStream::try_from(v.clone()).unwrap();
        assert_eq!(serde_json::to_value(&stream).unwrap(), v);
        let mut fields = stream.into_fields();
        fields
            .extensions
            .insert("stream_ref".into(), json!("false:identity"));
        assert!(ActuationStream::new(fields).is_err());
    }
    for invalid in [json!([]), json!(null), json!("stream"), json!(42)] {
        assert!(ActuationStream::try_from(invalid).is_err());
    }
}
#[test]
fn order_cursor_and_duplicate_laws_hold_for_many_contiguous_histories() {
    for length in 1..48u64 {
        let mut stream = opening().empty().unwrap();
        for n in 1..=length {
            stream = stream.append(event(n)).unwrap();
        }
        assert_eq!(stream.fields().cursor.fields().last_sequence.get(), length);
        for after in [0, length / 2, length, length + 1] {
            let page = stream.read(PageRequest {
                after_sequence: Count::new(after).unwrap(),
                limit: Some(Count::new(3).unwrap()),
            });
            assert!(page
                .events
                .iter()
                .all(|e| e.fields().sequence.get() > after));
            assert!(page.events.len() <= 3);
        }
        assert!(stream.append(event(length + 2)).is_err());
        let mut duplicate = event(length + 1).into_fields();
        duplicate.event_ref = event(1).fields().event_ref.clone();
        assert!(stream.append(StreamEvent::new(duplicate).unwrap()).is_err());
        let mut v = serde_json::to_value(&stream).unwrap();
        v["cursor"]["next_sequence"] = json!(length + 2);
        assert!(ActuationStream::try_from(v).is_err());
    }
}
#[test]
fn all_terminal_states_are_irreversible_without_erasing_return_or_dissent() {
    for terminal in [
        TerminalState::Closed,
        TerminalState::Interrupted,
        TerminalState::Cancelled,
    ] {
        let e: StreamEvent = decode(
            json!({"event_ref":"event:return","kind":"return","sequence":1,"return_ref":"return:dissent","content":"I disagree","actor":{"agent_ref":"agent:independent"}}),
        );
        let stream = opening()
            .empty()
            .unwrap()
            .append(e.clone())
            .unwrap()
            .close(terminal, Timestamp::new("2026-09-10T08:05:00Z").unwrap())
            .unwrap();
        assert_eq!(stream.fields().events, vec![e]);
        assert!(stream.append(event(2)).is_err());
        assert!(stream
            .close(terminal, Timestamp::new("2026-09-10T08:06:00Z").unwrap())
            .is_err());
        let v = serde_json::to_value(stream).unwrap();
        assert!(v.get("recognised").is_none());
        assert!(v.get("world_mutated").is_none());
    }
}
#[test]
fn unknown_and_unqueried_request_sides_remain_different() {
    let request = RequestRef::new("request:unknown").unwrap();
    assert_eq!(
        CorrelationCorpus::unqueried().correlate(&request)["state"],
        "correlation-unavailable"
    );
    let queried: CorrelationCorpus =
        decode(json!({"authority_decisions":[],"activities":[],"streams":[]}));
    assert_eq!(queried.correlate(&request)["state"], "unknown-identity");
    assert!(serde_json::from_value::<CorrelationCorpus>(json!({"activities":null})).is_err());
}
#[test]
fn timestamps_preserve_spelling_and_counts_cannot_escape_wire_precision() {
    let a = Timestamp::new("2026-09-10T09:00:00+01:00").unwrap();
    let b = Timestamp::new("2026-09-10T08:00:00Z").unwrap();
    assert_eq!(a.unix_nanos(), b.unix_nanos());
    assert_ne!(a.as_str(), b.as_str());
    assert_eq!(decode::<Count>(json!(1.0)), Count::ONE);
    for v in [
        json!(-1),
        json!(0.1),
        json!(9007199254740992u64),
        Value::Null,
        json!("1"),
    ] {
        assert!(serde_json::from_value::<Count>(v).is_err());
    }
    assert!(Count::new(9007199254740991).unwrap().next().is_err());
    assert!(Timestamp::new("not an observed timestamp").is_err());
}
#[test]
fn provider_absence_never_turns_into_identity_zero_counts_or_free_cost() {
    let value = serde_json::to_value(usage().observation).unwrap();
    for field in ["provider", "model"] {
        let mut v = value.clone();
        v[field] = json!({"standing":"not-reported","ref":"provider:invented"});
        assert!(ModelUsageObservation::try_from(v).is_err());
    }
    let mut v = value.clone();
    v["tokens"] = json!({"standing":"unavailable","input":0});
    assert!(ModelUsageObservation::try_from(v).is_err());
    let mut v = value.clone();
    v["cost"] = json!({"standing":"not-reported","amount":0,"currency":"USD"});
    assert!(ModelUsageObservation::try_from(v).is_err());
    let mut v = value.clone();
    v["provider_facts"] = json!({"arbitrary_payload":"secret"});
    assert!(ModelUsageObservation::try_from(v).is_err());
    let mut v = value;
    v["tokens"] = json!({"standing":"not-reported"});
    let admitted = ModelUsageObservation::try_from(v).unwrap();
    assert!(admitted.fields().tokens.fields().input.is_absent());
}
#[test]
fn reference_only_event_cannot_smuggle_native_content() {
    assert!(StreamEvent::try_from(json!({"event_ref":"event:native","kind":"world-observation","sequence":1,"disclosure":"reference-only","content":"private native payload"})).is_err());
    assert!(StreamEvent::try_from(json!({"event_ref":"event:native","kind":"world-observation","sequence":1,"disclosure":"reference-only","native_trace_ref":"trace:owner"})).is_ok());
}
#[test]
fn in_memory_subscription_replays_one_canonical_history_and_can_detach() {
    let mut journal = ActuationStreamJournal::new(opening().empty().unwrap());
    journal.append(event(1)).unwrap();
    let (id, rx) = journal.subscribe(Some(Count::ZERO), true).unwrap();
    assert_eq!(rx.try_recv().unwrap().event, event(1));
    journal.append(event(2)).unwrap();
    let item = rx.try_recv().unwrap();
    assert_eq!(item.event, event(2));
    assert_eq!(item.snapshot.events.len(), 2);
    assert!(journal.unsubscribe(id));
    journal.append(event(3)).unwrap();
    assert!(rx.try_recv().is_err());
    let (_, rx) = journal.subscribe(Some(Count::ONE), true).unwrap();
    assert_eq!(rx.try_recv().unwrap().event, event(2));
    assert_eq!(rx.try_recv().unwrap().event, event(3));
    journal
        .close(
            TerminalState::Closed,
            Timestamp::new("2026-09-10T09:00:00Z").unwrap(),
        )
        .unwrap();
    assert!(journal.append(event(4)).is_err());
}
#[test]
fn durable_append_keeps_prior_bytes_and_terminal_change_keeps_raw_event_tail() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    let o = opening();
    store.open(&o).unwrap();
    let header = fs::read(store.path(&o.stream_ref)).unwrap();
    store.append(&o.stream_ref, event(1)).unwrap();
    let first = fs::read(store.path(&o.stream_ref)).unwrap();
    assert!(first.starts_with(&header));
    // Whitespace and foreign extension formatting are evidence bytes, not something a lifecycle operation normalises.
    let raw = String::from_utf8(first)
        .unwrap()
        .replace("\"content\":", "\"content\" : ");
    fs::write(store.path(&o.stream_ref), &raw).unwrap();
    let tail = raw.split_once('\n').unwrap().1;
    store
        .close(
            &o.stream_ref,
            TerminalState::Cancelled,
            Timestamp::new("2026-09-10T09:00:00Z").unwrap(),
        )
        .unwrap();
    assert_eq!(
        fs::read_to_string(store.path(&o.stream_ref))
            .unwrap()
            .split_once('\n')
            .unwrap()
            .1,
        tail
    );
}
#[test]
fn valid_unterminated_last_record_gets_a_separator_not_a_corrupt_concatenation() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    let o = opening();
    store.open(&o).unwrap();
    let raw = fs::read_to_string(store.path(&o.stream_ref))
        .unwrap()
        .trim_end_matches('\n')
        .to_owned();
    fs::write(store.path(&o.stream_ref), &raw).unwrap();
    assert!(store.load(&o.stream_ref).is_ok());
    store.append(&o.stream_ref, event(1)).unwrap();
    let next = fs::read_to_string(store.path(&o.stream_ref)).unwrap();
    assert!(next.starts_with(&raw));
    assert_eq!(store.load(&o.stream_ref).unwrap().fields().events.len(), 1);
}
#[test]
fn torn_tail_holes_bad_sequence_and_identity_refuse_without_rewriting() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    let o = opening();
    store.open(&o).unwrap();
    let header = fs::read_to_string(store.path(&o.stream_ref)).unwrap();
    for suffix in [
        "{\"event_ref\":",
        "\n",
        "{\"event_ref\":\"event:x\",\"sequence\":2,\"kind\":\"world-observation\"}\n",
    ] {
        let raw = format!("{header}{suffix}");
        fs::write(store.path(&o.stream_ref), &raw).unwrap();
        assert!(store.load(&o.stream_ref).is_err());
        assert!(store.append(&o.stream_ref, event(1)).is_err());
        assert_eq!(fs::read_to_string(store.path(&o.stream_ref)).unwrap(), raw);
    }
    fs::write(store.path(&o.stream_ref), &header).unwrap();
    let mut conflict = opening();
    conflict.agency_ref = AgencyRef::new("agency:other").unwrap();
    assert!(store.open(&conflict).is_err());
    assert_eq!(
        fs::read_to_string(store.path(&o.stream_ref)).unwrap(),
        header
    );
}
#[test]
fn idempotent_usage_survives_close_but_conflicting_usage_and_wrong_correlation_refuse() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    let o = opening();
    store.open(&o).unwrap();
    let u = usage();
    let first = store.record_usage(u.clone()).unwrap();
    assert_eq!(first.deduplicated, Some(false));
    store
        .close(
            &o.stream_ref,
            TerminalState::Closed,
            Timestamp::new("2026-09-10T09:00:00Z").unwrap(),
        )
        .unwrap();
    let closed = fs::read(store.path(&o.stream_ref)).unwrap();
    let replay = store.record_usage(u.clone()).unwrap();
    assert_eq!(replay.deduplicated, Some(true));
    assert_eq!(replay.event, first.event);
    let mut v = serde_json::to_value(&u.observation).unwrap();
    v["tokens"]["output"] = json!(999);
    let mut conflict = u.clone();
    conflict.observation = decode(v);
    assert!(store.record_usage(conflict).is_err());
    let mut v = serde_json::to_value(&u.observation).unwrap();
    v["correlation"]["agency_ref"] = json!("agency:other");
    let mut conflict = u;
    conflict.observation = decode(v);
    assert!(store.record_usage(conflict).is_err());
    assert_eq!(fs::read(store.path(&o.stream_ref)).unwrap(), closed);
}
#[test]
fn filename_encoding_cannot_traverse_and_mismatched_or_symlinked_stores_refuse() {
    let o = opening();
    let name = stream_file_name(&o.stream_ref);
    assert!(!name.contains('/'));
    assert!(name.contains("%E9%9B%AA"));
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    store.open(&o).unwrap();
    let other = StreamRef::new("../../other").unwrap();
    fs::copy(store.path(&o.stream_ref), store.path(&other)).unwrap();
    assert!(store.load(&other).is_err());
    #[cfg(unix)]
    {
        fs::remove_file(store.path(&other)).unwrap();
        std::os::unix::fs::symlink(store.path(&o.stream_ref), store.path(&other)).unwrap();
        assert!(store.load(&other).is_err());
        assert!(store.append(&other, event(1)).is_err());
    }
}
#[test]
fn independent_native_writers_share_one_atomic_contiguous_sequence() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    store.open(&opening()).unwrap();
    let barrier = Arc::new(Barrier::new(12));
    let mut handles = vec![];
    for index in 0..12 {
        let store = store.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            barrier.wait();
            store
                .record_boundary(boundary(index), &support::FixtureCatalogue::new())
                .unwrap()
        }));
    }
    let mut sequences: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().unwrap().event.fields().sequence.get())
        .collect();
    sequences.sort();
    assert_eq!(sequences, (1..=12).collect::<Vec<_>>());
    assert_eq!(
        store
            .load(&opening().stream_ref)
            .unwrap()
            .fields()
            .events
            .len(),
        12
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}
#[test]
fn unrelated_stream_and_agent_continue_after_another_stream_terminates() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    let one = opening();
    let mut two = opening();
    two.stream_ref = StreamRef::new("stream:independent").unwrap();
    two.agency_ref = AgencyRef::new("agency:independent").unwrap();
    two.actuation_ref = ActuationRef::new("act:independent").unwrap();
    store.open(&one).unwrap();
    store.open(&two).unwrap();
    store
        .close(
            &one.stream_ref,
            TerminalState::Cancelled,
            Timestamp::new("2026-09-10T09:00:00Z").unwrap(),
        )
        .unwrap();
    let next = store.append(&two.stream_ref, event(1)).unwrap();
    assert_eq!(next.fields().agency_ref, two.agency_ref);
    assert_eq!(next.fields().lifecycle.fields().state, StreamState::Open);
}
#[test]
fn forged_catalogue_answer_cannot_relabel_an_observed_native_occurrence() {
    struct Wrong;
    impl BoundaryCatalogue for Wrong {
        fn boundary(&self, _: &str, _: &str) -> Result<NativeBoundary> {
            Ok(NativeBoundary {
                harness: ExternalRef::new("different")?,
                native_event: ExternalRef::new("Different")?,
                boundary: None,
                catalog_revision: None,
            })
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(dir.path()).unwrap();
    assert!(store.record_boundary(boundary(1), &Wrong).is_err());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
