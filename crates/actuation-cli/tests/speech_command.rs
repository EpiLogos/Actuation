//! The speech/Nara verbs through the real dispatch table: document in, native
//! receipt document out. No server process, no network, no audio material.

use actuation_cli::execute;
use serde_json::{json, Value};

const CONSTITUTION_VERSION: &str = "actuation.speech-constitution/v1";
const DECISION_VERSION: &str = "actuation.speech-tool-decision/v1";
const INTERRUPTION_VERSION: &str = "actuation.speech-interruption/v1";
const SESSION_READ_VERSION: &str = "actuation.speech-session-read/v1";
const CONTEXT_READ_VERSION: &str = "actuation.nara-context-read/v1";
const DELEGATION_RECEIPT_VERSION: &str = "actuation.nara-delegation/v1";
const ENRICHMENT_RECEIPT_VERSION: &str = "actuation.nara-enrichment/v1";

fn argv(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn constitution_value(body: &str) -> Value {
    json!({
        "schema": CONSTITUTION_VERSION,
        "constitution_ref": format!("constitution:{body}"),
        "agent_ref": "nara:canonical",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:nara",
        "agent_session_ref": "session:nara-1",
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
        "interruption": {"state": "supported"},
        "provider_binding": {"provider_ref": format!("provider:{body}"),
            "provider_session_ref": format!("provider-session:{body}")},
        "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
        "resolved_at": "2026-09-18T09:00:00Z"
    })
}

fn text_only_constitution_value() -> Value {
    json!({
        "schema": CONSTITUTION_VERSION,
        "constitution_ref": "constitution:text",
        "agent_ref": "nara:canonical",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:nara",
        "agent_session_ref": "session:nara-1",
        "body_ref": "model-surface:text",
        "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:opaque",
            "inference_surface": {"contract_ref": "contract:opaque"}},
        "access_profile": {"schema": "actuation.instantiation/v1",
            "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
        "input_modalities": ["text"],
        "output_modalities": ["text"],
        "interaction": {"request-response": {"state": "supported"},
            "barge-in": {"state": "unsupported", "reason": "no speech output exists to interrupt"}},
        "transport": "http",
        "connection": {"kind": "stateless", "reconnect": null},
        "interruption": {"state": "unsupported", "reason": "no speech output exists to interrupt"},
        "provider_binding": {"provider_ref": "provider:text"},
        "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
        "resolved_at": "2026-09-18T09:00:00Z"
    })
}

fn context_value() -> Value {
    json!({
        "schema": "ql.nara-dialogue-context/v1",
        "context_ref": "context:encounter-1",
        "nara_ref": "nara:canonical",
        "subject_ref": "subject:frank",
        "agent_session_ref": "session:nara-1",
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

fn delegation_value() -> Value {
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

fn enrichment_value() -> Value {
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

fn run(route: &[&str], stdin: &Value) -> Value {
    let mut command = argv(route);
    command.push("--json".into());
    let output = execute(&command, &stdin.to_string()).expect("speech verbs must not error");
    assert_eq!(output.code, 0, "typed refusals still exit cleanly");
    serde_json::from_str(&output.stdout).expect("output is JSON")
}

fn run_refused(route: &[&str], stdin: &Value) -> String {
    let mut command = argv(route);
    command.push("--json".into());
    execute(&command, &stdin.to_string())
        .expect_err("the verb must refuse")
        .to_string()
}

#[test]
fn speech_constitution_admits_and_reprints_the_document() {
    let admitted = run(
        &["speech", "constitution", "-"],
        &constitution_value("realtime"),
    );
    assert_eq!(admitted["schema"], CONSTITUTION_VERSION);
    assert_eq!(admitted["body_ref"], "model-surface:realtime");
    // Foreign fields survive: the verb prints the admitted document itself.
    let mut legacy = text_only_constitution_value();
    legacy["legacy_native_note"] = json!("kept verbatim");
    let admitted = run(&["speech", "constitution", "-"], &legacy);
    assert_eq!(admitted["legacy_native_note"], "kept verbatim");
    // An invented vocabulary term is refused with a semantic error.
    let mut invented = constitution_value("realtime");
    invented["input_modalities"] = json!(["braille"]);
    let error = run_refused(&["speech", "constitution", "-"], &invented);
    assert!(error.contains("unknown modality"), "{error}");
}

#[test]
fn speech_decision_runs_the_authority_gate_and_prints_the_receipt() {
    let input = json!({
        "constitution": constitution_value("realtime"),
        "request": {
            "schema": DECISION_VERSION,
            "request_ref": "request:model-1",
            "constitution_ref": "constitution:realtime",
            "agent_session_ref": "session:nara-1",
            "proposed_action_ref": "action:read-coordinate",
            "payload_refs": ["ref:spoken-target"],
            "requested_at": "2026-09-18T09:01:00Z"
        },
        "allowed_action_refs": ["action:read-coordinate"],
        "denied_action_refs": ["action:delete-world"],
        "decision_ref": "decision:allowed",
        "decided_by": "human:owner",
        "decided_at": "2026-09-18T09:01:01Z"
    });
    let receipt = run(&["speech", "decision", "-"], &input);
    assert_eq!(receipt["schema"], DECISION_VERSION);
    assert_eq!(receipt["resolution"]["resolution"], "authorised");
    assert!(receipt["execution"].is_null(), "authorised is not executed");

    // An unlisted action is refused: available is not authorised.
    let mut unlisted = input.clone();
    unlisted["request"]["proposed_action_ref"] = json!("action:not-in-the-list");
    unlisted["decision_ref"] = json!("decision:unlisted");
    let receipt = run(&["speech", "decision", "-"], &unlisted);
    assert_eq!(receipt["resolution"]["resolution"], "refused");
    assert_eq!(receipt["resolution"]["stage"], "unauthorised");
}

#[test]
fn speech_interrupt_reconstitutes_the_described_state_and_prints_the_receipt() {
    let input = json!({
        "constitution": constitution_value("realtime"),
        "session_state": {
            "phase": "speaking",
            "in_flight_response_ref": "response:1",
            "executed_refs": ["result:done"]
        },
        "interruption_ref": "interruption:1",
        "reason": "user barge-in",
        "at": "2026-09-18T09:02:00Z"
    });
    let receipt = run(&["speech", "interrupt", "-"], &input);
    assert_eq!(receipt["schema"], INTERRUPTION_VERSION);
    assert_eq!(receipt["outcome"], "cancelled");
    assert_eq!(receipt["response_ref"], "response:1");
    assert_eq!(receipt["executed_refs"], json!(["result:done"]));
    assert_eq!(receipt["agent_ref"], "nara:canonical");

    // Where the body cannot cancel, the receipt refuses honestly and the
    // in-flight response stands.
    let mut no_cancel = constitution_value("batch");
    no_cancel["interruption"] =
        json!({"state": "unsupported", "reason": "the provider cannot cancel mid-utterance"});
    let refused_input = json!({
        "constitution": no_cancel,
        "session_state": {
            "phase": "speaking",
            "in_flight_response_ref": "response:2",
            "executed_refs": []
        },
        "interruption_ref": "interruption:2",
        "reason": "stop",
        "at": "2026-09-18T09:03:00Z"
    });
    let receipt = run(&["speech", "interrupt", "-"], &refused_input);
    assert_eq!(receipt["outcome"], "refused");
    assert_eq!(receipt["response_ref"], "response:2");
}

#[test]
fn speech_session_prints_the_read_model_of_a_described_session() {
    let input = json!({
        "constitution": constitution_value("realtime"),
        "session_state": {"phase": "listening"},
        "interruption_ref": null
    });
    let read = run(&["speech", "session", "-"], &input);
    assert_eq!(read["schema"], SESSION_READ_VERSION);
    assert_eq!(read["phase"], "listening");
    assert_eq!(read["speech_capable"], true);
    assert_eq!(read["realtime_capable"], true);

    // A text-only body can be described as text-only: the read model says so.
    let text_only = json!({
        "constitution": text_only_constitution_value(),
        "session_state": null,
        "interruption_ref": null
    });
    let read = run(&["speech", "session", "-"], &text_only);
    assert_eq!(read["speech_capable"], false);
    assert_eq!(read["phase"], "idle");

    // But it cannot be described into listening: a state the session could
    // not actually be in is refused, not approximated.
    let impossible = json!({
        "constitution": text_only_constitution_value(),
        "session_state": {"phase": "listening"},
        "interruption_ref": null
    });
    let error = run_refused(&["speech", "session", "-"], &impossible);
    assert!(
        error.contains("cannot hear speech"),
        "a text-only body cannot be described into listening: {error}"
    );
}

#[test]
fn nara_context_admits_a_bounded_dialogue_context() {
    let input = json!({
        "constitution": constitution_value("realtime"),
        "context": context_value()
    });
    let reading = run(&["nara", "context", "-"], &input);
    assert_eq!(reading["schema"], CONTEXT_READ_VERSION);
    assert_eq!(reading["nara_ref"], "nara:canonical");
    assert_eq!(reading["expressive_act"]["live"], true);

    // A foreign Nara's context is refused.
    let mut foreign = input.clone();
    foreign["context"]["nara_ref"] = json!("nara:other");
    let error = run_refused(&["nara", "context", "-"], &foreign);
    assert!(error.contains("another Nara"), "{error}");
}

#[test]
fn nara_delegate_and_enrichment_print_their_receipts() {
    let delegate_input = json!({
        "constitution": constitution_value("realtime"),
        "context": context_value(),
        "delegation": delegation_value(),
        "delegation_receipt_ref": "delegation-receipt:1",
        "at": "2026-09-18T09:04:00Z"
    });
    let receipt = run(&["nara", "delegate", "-"], &delegate_input);
    assert_eq!(receipt["schema"], DELEGATION_RECEIPT_VERSION);
    assert_eq!(receipt["foreground_agent"], "nara:canonical");
    assert_eq!(receipt["basis_current_at_receipt"], true);

    // A current enrichment is proposed only; nothing auto-applies.
    let enrichment_input = json!({
        "constitution": constitution_value("realtime"),
        "context": context_value(),
        "delegation": delegation_value(),
        "delegation_receipt_ref": "delegation-receipt:2",
        "delegated_at": "2026-09-18T09:05:00Z",
        "enrichment": enrichment_value(),
        "enrichment_receipt_ref": "enrichment-receipt:1",
        "at": "2026-09-18T09:06:00Z"
    });
    let receipt = run(&["nara", "enrichment", "-"], &enrichment_input);
    assert_eq!(receipt["schema"], ENRICHMENT_RECEIPT_VERSION);
    assert_eq!(receipt["standing"], "proposed-only");
    assert_eq!(receipt["applied"], false);

    // A stale enrichment is retained, never applied.
    let mut stale_enrichment = enrichment_input;
    stale_enrichment["enrichment"]["basis_expression_revision"] = json!("rev-5");
    stale_enrichment["enrichment_receipt_ref"] = json!("enrichment-receipt:2");
    let receipt = run(&["nara", "enrichment", "-"], &stale_enrichment);
    assert_eq!(receipt["standing"], "retained-not-applied");
    assert_eq!(receipt["applied"], false);
}

#[test]
fn bare_speech_and_nara_name_their_subcommands() {
    for (bare, expected) in [
        (
            "speech",
            "expected constitution, decision, interrupt or session",
        ),
        ("nara", "expected context, delegate or enrichment"),
    ] {
        let error = execute(&argv(&[bare]), "").unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}
