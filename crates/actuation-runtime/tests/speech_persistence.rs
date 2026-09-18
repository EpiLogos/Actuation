//! Persistence proof for the speech/Nara lane (Actuation #94/#91 follow-up).
//!
//! Every receipt the lane produces — constitution, constitution change, tool
//! decision, generic and Nara interruption, delegation, enrichment — is
//! produced through the real binding API, appended to a real
//! `JsonlStreamStore` on disk, and then read back by a *fresh* store
//! instance over the same directory. The replayed documents must equal the
//! originals byte-for-byte (serialized string equality, which the
//! order-preserving JSON backend makes a genuine byte test) and must still
//! admit through their own validators.

use actuation_adapters::{
    validate_speech_constitution, validate_speech_constitution_change,
    validate_speech_tool_decision, SpeechConstitution, SpeechToolRequest,
    SPEECH_CONSTITUTION_VERSION, SPEECH_TOOL_DECISION_VERSION,
};
use actuation_core::{ActuationRef, AgencyRef, AgentSessionRef, ExternalRef, StreamRef};
use actuation_runtime::{
    validate_nara_interruption, validate_speech_interruption, NaraBinding, SpeechLaneReceipt,
    SpeechReceiptStream, SpeechSession, NARA_DELEGATION_VERSION, NARA_ENRICHMENT_VERSION,
};
use actuation_stream::{JsonlStreamStore, OpenStream, PageRequest, StreamStore};
use serde_json::{json, Value};

fn constitution(body: &str, agent_session: &str, interruption: Value) -> SpeechConstitution {
    SpeechConstitution::try_from(json!({
        "schema": SPEECH_CONSTITUTION_VERSION,
        "constitution_ref": format!("constitution:{body}"),
        "agent_ref": "nara:canonical",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:nara",
        "agent_session_ref": agent_session,
        "body_ref": format!("model-surface:{body}"),
        "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:opaque",
            "inference_surface": {"contract_ref": "contract:opaque"}},
        "access_profile": {"schema": "actuation.instantiation/v1",
            "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
        "input_modalities": ["text", "speech"],
        "output_modalities": ["text", "speech"],
        "transforms": {"speech-to-speech": {"state": "supported"}},
        "interaction": {
            "request-response": {"state": "supported"},
            "full-duplex-realtime": {"state": "supported"},
            "tool-requests": {"state": "supported"}
        },
        "transport": "websocket",
        "connection": {"kind": "connected", "reconnect": "resumable"},
        "interruption": interruption,
        "provider_binding": {"provider_ref": format!("provider:{body}"),
            "provider_session_ref": format!("provider-session:{body}")},
        "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
        "resolved_at": "2026-09-18T09:00:00Z"
    }))
    .expect("fixture constitution must be valid")
}

fn context(agent_session: &str) -> Value {
    json!({
        "schema": "ql.nara-dialogue-context/v1",
        "context_ref": "context:encounter-1",
        "nara_ref": "nara:canonical",
        "subject_ref": "subject:frank",
        "agent_session_ref": agent_session,
        "coordinate_ref": "M2-5-9:C0",
        "expression_ref": "expression:1",
        "expression_revision": "rev-7",
        "profile_ref": "profile:1",
        "profile_revision": "rev-2",
        "active_m_focus": "m4",
        "disclosed": [],
        "available_action_refs": [],
        "expressive_act": {
            "expressive_act_ref": "expressive-act:9",
            "phase": "active",
            "basis_expression_revision": "rev-7",
            "speech_turn_ref": "response:1"
        }
    })
}

fn delegation() -> Value {
    json!({
        "schema": "ql.nara-epii-delegation/v1",
        "delegation_ref": "delegation:1",
        "nara_ref": "nara:canonical",
        "epii_session_ref": "session:epii-1",
        "basis": {
            "context_ref": "context:encounter-1",
            "expression_ref": "expression:1",
            "expression_revision": "rev-7",
            "profile_ref": "profile:1",
            "profile_revision": "rev-2",
            "coordinate_ref": "M2-5-9:C0",
            "bimba_registry_revision": "registry-3"
        },
        "brief": "resolve the relation under focus",
        "scope_refs": ["source:1"],
        "delegated_at_unix_ms": 1,
        "state": "delegated"
    })
}

fn enrichment() -> Value {
    json!({
        "schema": "ql.epii-enrichment/v1",
        "enrichment_ref": "enrichment:1",
        "delegation_ref": "delegation:1",
        "basis_context_ref": "context:encounter-1",
        "basis_expression_revision": "rev-7",
        "proposed_focus_refs": ["source:1"],
        "coordinate_refs": [],
        "summary": "the relation resolves as ..."
    })
}

fn serialized(value: &Value) -> String {
    serde_json::to_string(value).expect("receipt documents are JSON")
}

fn open_stream(root: &std::path::Path, stream_ref: &StreamRef) -> JsonlStreamStore {
    let store = JsonlStreamStore::new(root).unwrap();
    store
        .open(&OpenStream {
            stream_ref: stream_ref.clone(),
            actuation_ref: ActuationRef::new("actuation:nara-1").unwrap(),
            agency_ref: AgencyRef::new("agency:nara").unwrap(),
            agent_session_ref: AgentSessionRef::new("session:nara-1").unwrap(),
            world_binding_ref: None,
            provenance: None,
            started_at: None,
        })
        .unwrap();
    store
}

#[test]
fn the_lane_receipts_persist_and_replay_byte_faithfully_through_a_fresh_store() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let stream_ref = StreamRef::new("stream:speech-nara").unwrap();
    let _ = open_stream(root, &stream_ref);

    // One lane walk with a real NaraBinding: every receipt kind produced
    // through the real API. All documents stay in scope for the whole test;
    // the typed wrappers borrow them.
    let realtime = constitution("realtime", "session:nara-1", json!({"state": "supported"}));
    let mut nara = NaraBinding::constitute(
        realtime.clone(),
        context("session:nara-1"),
        vec![ExternalRef::new("action:read-coordinate").unwrap()],
        vec![ExternalRef::new("action:delete-world").unwrap()],
    )
    .unwrap();

    // A decision: refused (the action is not in the allowed list) — refusals
    // are recorded exactly like authorisations.
    let request = SpeechToolRequest::try_from(json!({
        "schema": SPEECH_TOOL_DECISION_VERSION,
        "request_ref": "request:model-1",
        "constitution_ref": "constitution:realtime",
        "agent_session_ref": "session:nara-1",
        "proposed_action_ref": "action:not-in-the-list",
        "payload_refs": ["ref:spoken-target"],
        "requested_at": "2026-09-18T09:01:00Z"
    }))
    .unwrap();
    let decision = nara
        .adjudicate_tool_request("decision:1", request, "human:owner", "2026-09-18T09:01:01Z")
        .unwrap();
    assert!(!decision.is_authorised());

    // An interruption with a live ExpressiveAct correlation.
    nara.session_mut()
        .begin_response(ExternalRef::new("response:1").unwrap())
        .unwrap();
    nara.session_mut()
        .commit_result(ExternalRef::new("result:done").unwrap())
        .unwrap();
    let nara_interruption = nara
        .interrupt("interruption:1", "user barge-in", "2026-09-18T09:02:00Z")
        .unwrap();
    assert!(nara_interruption.cancelled());

    // Delegation and its (current) enrichment.
    let delegation_receipt = nara
        .delegate_to_epii(delegation(), "delegation-receipt:1", "2026-09-18T09:03:00Z")
        .unwrap();
    let enrichment_receipt = nara
        .receive_enrichment(enrichment(), "enrichment-receipt:1", "2026-09-18T09:04:00Z")
        .unwrap();
    assert_eq!(enrichment_receipt["standing"], "proposed-only");

    // A body swap: realtime -> cascade on the same identity.
    let change = nara
        .reconnect(
            "change:rt-to-cascade",
            constitution("cascade", "session:nara-1", json!({"state": "supported"})),
            context("session:nara-1"),
            "realtime provider lost; cascade fallback",
            vec![ExternalRef::new("evidence:outage").unwrap()],
            "2026-09-18T09:05:00Z",
        )
        .unwrap();

    // A later generic interruption receipt from a fresh session.
    let mut session = SpeechSession::constitute(constitution(
        "cascade-2",
        "session:nara-1",
        json!({"state": "supported"}),
    ))
    .unwrap();
    session
        .begin_response(ExternalRef::new("response:2").unwrap())
        .unwrap();
    let generic_interrupted = session.interrupt("interruption:2", "stop", "2026-09-18T09:06:00Z");

    let lanes: [(&str, SpeechLaneReceipt, &dyn Fn(&Value) -> bool); 7] = [
        (
            "actuation:event:constitution",
            SpeechLaneReceipt::Constitution(&realtime),
            &|v| validate_speech_constitution(v).is_ok(),
        ),
        (
            "actuation:event:decision",
            SpeechLaneReceipt::ToolDecision(&decision),
            &|v| validate_speech_tool_decision(v).is_ok(),
        ),
        (
            "actuation:event:nara-interruption",
            SpeechLaneReceipt::NaraInterruption(&nara_interruption),
            &|v| validate_nara_interruption(v).is_ok(),
        ),
        (
            "actuation:event:delegation",
            SpeechLaneReceipt::NaraDelegation(&delegation_receipt),
            &|v| v["schema"] == json!(NARA_DELEGATION_VERSION),
        ),
        (
            "actuation:event:enrichment",
            SpeechLaneReceipt::NaraEnrichment(&enrichment_receipt),
            &|v| v["schema"] == json!(NARA_ENRICHMENT_VERSION),
        ),
        (
            "actuation:event:change",
            SpeechLaneReceipt::ConstitutionChange(&change),
            &|v| validate_speech_constitution_change(v).is_ok(),
        ),
        (
            "actuation:event:interruption",
            SpeechLaneReceipt::Interruption(&generic_interrupted),
            &|v| validate_speech_interruption(v).is_ok(),
        ),
    ];

    // Record every receipt through the real recorder.
    let mut recorder = SpeechReceiptStream::new(
        open_stream(root, &stream_ref),
        stream_ref.clone(),
        ActuationRef::new("actuation:nara-1").unwrap(),
    );
    let mut originals: Vec<(String, String, Value)> = Vec::new();
    for (event_ref, receipt, _) in &lanes {
        let schema = receipt.schema().unwrap().to_owned();
        let document = receipt.document().unwrap();
        recorder.record(*event_ref, receipt.clone()).unwrap();
        originals.push(((*event_ref).to_owned(), schema, document));
    }

    // A fresh store instance over the same directory: everything below reads
    // the durable bytes, never in-memory state.
    let reread = JsonlStreamStore::new(root).unwrap();
    let page = reread.replay(&stream_ref, PageRequest::default()).unwrap();
    assert_eq!(
        page.events.len(),
        originals.len(),
        "every recorded receipt must come back from disk"
    );

    for (event, (event_ref, schema, original)) in page.events.iter().zip(&originals) {
        assert_eq!(event.fields().event_ref.as_str(), event_ref.as_str());
        // The event's custom kind is the receipt's own schema version.
        assert_eq!(
            event.fields().custom_kind.value().map(|k| k.as_str()),
            Some(schema.as_str()),
            "event {event_ref} must carry its receipt schema as its custom kind"
        );
        let metadata = event.fields().metadata.value().expect("receipt metadata");
        assert_eq!(
            metadata.get("receipt_schema").and_then(Value::as_str),
            Some(schema.as_str())
        );
        // Byte-faithful: the serialized replayed document equals the
        // serialized original exactly, key order included.
        let replayed = metadata.get("receipt").expect("receipt document");
        assert_eq!(
            serialized(replayed),
            serialized(original),
            "receipt {event_ref} must replay byte-faithfully"
        );
    }

    // Each replayed document still admits through its own validator, in the
    // same order the lanes were recorded.
    for (event, (event_ref, _, revalidate)) in page.events.iter().zip(&lanes) {
        let receipt = &event.fields().metadata.value().unwrap()["receipt"];
        assert!(
            revalidate(receipt),
            "receipt {event_ref} must still admit after replay"
        );
    }
}

#[test]
fn the_durable_file_folds_back_into_the_same_stream() {
    // The appended receipts survive the fold law: reading the raw JSONL file
    // reconstructs the identical stream, attributed from the receipt itself.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let stream_ref = StreamRef::new("stream:speech-fold").unwrap();
    let _ = open_stream(root, &stream_ref);
    let realtime = constitution("realtime", "session:nara-1", json!({"state": "supported"}));
    let mut recorder = SpeechReceiptStream::new(
        open_stream(root, &stream_ref),
        stream_ref.clone(),
        ActuationRef::new("actuation:nara-1").unwrap(),
    );
    recorder
        .record(
            "actuation:event:fold-1",
            SpeechLaneReceipt::Constitution(&realtime),
        )
        .unwrap();

    let raw =
        std::fs::read_to_string(JsonlStreamStore::new(root).unwrap().path(&stream_ref)).unwrap();
    let folded = actuation_stream::fold_stream_file(&raw).unwrap();
    assert_eq!(folded.fields().events.len(), 1);
    let metadata = folded.fields().events[0].fields().metadata.value().unwrap();
    assert_eq!(
        serialized(&metadata["receipt"]),
        serialized(realtime.as_value())
    );
    // Attribution came from the receipt itself.
    let actor = folded.fields().events[0].fields().actor.value().unwrap();
    assert_eq!(
        actor.fields().agent_ref.value().map(|a| a.as_str()),
        Some("nara:canonical")
    );
}
