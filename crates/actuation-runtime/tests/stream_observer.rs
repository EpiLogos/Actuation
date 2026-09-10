#[allow(dead_code)]
#[path = "support/runtime.rs"]
mod runtime;
use actuation_core::*;
use actuation_runtime::*;
use actuation_stream::*;
use serde_json::{json, Value};
fn setup() -> (tempfile::TempDir, JsonlStreamStore, OpenStream, Attribution) {
    let root = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(root.path()).unwrap();
    let opening:OpenStream=serde_json::from_value(json!({"stream_ref":"stream:direct","actuation_ref":"act:direct","agency_ref":"agency:direct","agent_session_ref":"session:external","world_binding_ref":"binding:project"})).unwrap();
    store.open(&opening).unwrap();
    let actor =
        serde_json::from_value(json!({"agent_ref":"agent:direct","agency_ref":"agency:direct"}))
            .unwrap();
    (root, store, opening, actor)
}
#[test]
fn ordinary_loop_evidence_is_replayable_actuality_without_factory_ancestry() {
    let (_root, store, o, actor) = setup();
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/runtime.json")).unwrap();
    let input = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "loop")
        .unwrap()["input"]
        .clone();
    let request = LoopRequest::from_legacy(input["request"].clone()).unwrap();
    let mut host = runtime::ScriptedHost::new(&input);
    let mut observer = StreamRuntimeObserver::with_clock(
        store,
        o.stream_ref.clone(),
        o.actuation_ref.clone(),
        actor,
        || Timestamp::new("2026-09-10T08:00:00Z"),
    );
    let execution = runtime::block_on(ClassicRuntime.run(
        &request,
        &mut host,
        &mut observer,
        &CancellationToken::default(),
    ))
    .unwrap();
    let stream = observer.store().load(&o.stream_ref).unwrap();
    assert!(!stream.fields().events.is_empty());
    assert_eq!(execution.evidence_refs.len(), stream.fields().events.len());
    for (reference, event) in execution.evidence_refs.iter().zip(&stream.fields().events) {
        assert_eq!(reference.as_str(), event.fields().event_ref.as_str());
        let native = event
            .fields()
            .metadata
            .value()
            .unwrap()
            .get("runtime_event")
            .unwrap();
        assert_eq!(native["event_id"], reference.as_str());
        assert!(native.get("payload").is_some());
        let wire = serde_json::to_value(event).unwrap();
        assert!(wire.get("run_ref").is_none());
        assert!(wire.get("recognition_ref").is_none());
    }
}
#[test]
fn observer_never_returns_an_evidence_ref_after_identity_or_persistence_refusal() {
    let (_root, store, o, actor) = setup();
    let event = LoopEvent {
        channel: "runtime".into(),
        event_id: "event:actual".into(),
        event_type: "model.returned".into(),
        run_id: ExternalRef::new("loop:trace").unwrap(),
        sequence: 1,
        runtime: "classic".into(),
        payload: json!({"status":"completed"}),
    };
    let mut observer = StreamRuntimeObserver::new(
        store.clone(),
        o.stream_ref.clone(),
        ActuationRef::new("act:other").unwrap(),
        actor.clone(),
    );
    assert!(observer.emit(&event).is_err());
    assert!(store
        .load(&o.stream_ref)
        .unwrap()
        .fields()
        .events
        .is_empty());
    store
        .close(
            &o.stream_ref,
            TerminalState::Closed,
            Timestamp::new("2026-09-10T08:00:00Z").unwrap(),
        )
        .unwrap();
    let mut observer = StreamRuntimeObserver::new(store, o.stream_ref, o.actuation_ref, actor);
    assert!(observer.emit(&event).is_err());
}
