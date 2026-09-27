//! Actual CLI process/lifecycle tests. Constitutions below are contract specimens,
//! not evidence of a live microphone, provider, speaker or full voice floor.
use actuation_adapters::{SpeechConstitution, SPEECH_CONSTITUTION_VERSION};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

#[test]
fn actual_capabilities_preserve_the_frozen_surface_and_add_only_nara_serve() {
    let output = Command::new(env!("CARGO_BIN_EXE_actuation"))
        .args(["capabilities", "--json"])
        .output()
        .expect("actual CLI must start");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let mut actual: Value = serde_json::from_slice(&output.stdout).unwrap();
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/scenarios.json")).unwrap();
    let mut expected = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "cli-capabilities --json")
        .unwrap()["expected"]["stdout"]
        .clone();
    expected["commands"]
        .as_array_mut()
        .unwrap()
        .insert(0, json!("nara.serve"));
    actual["revision"] = json!("$REVISION");
    assert_eq!(
        actual, expected,
        "The new actor must not drift the existing CLI contract"
    );
}

fn constitution(
    body: &str,
    agent_session: &str,
    speech: bool,
    interruption: Value,
) -> SpeechConstitution {
    let (input, output, interaction, transport, connection) = if speech {
        (
            json!(["text", "speech"]),
            json!(["text", "speech"]),
            json!({
                "request-response": {"state": "supported"},
                "streaming-input": {"state": "supported"},
                "streaming-output": {"state": "supported"},
                "full-duplex-realtime": {"state": "supported"},
                "structured-events": {"state": "supported"},
                "tool-requests": {"state": "supported"},
                "partial-transcripts": {"state": "supported"},
                "final-transcripts": {"state": "supported"},
                "vad-turn-detection": {"state": "supported"},
                "barge-in": {"state": "supported"}
            }),
            json!("websocket"),
            json!({"kind": "connected", "reconnect": "resumable"}),
        )
    } else {
        (
            json!(["text"]),
            json!(["text"]),
            json!({
                "request-response": {"state": "supported"},
                "tool-requests": {"state": "supported"},
                "barge-in": {"state": "unsupported", "reason": "no speech output exists to interrupt"}
            }),
            json!("http"),
            json!({"kind": "stateless", "reconnect": null}),
        )
    };
    SpeechConstitution::try_from(json!({
        "schema": SPEECH_CONSTITUTION_VERSION,
        "constitution_ref": format!("constitution:{body}"),
        "agent_ref": "nara:canonical",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:shared-world",
        "agent_session_ref": agent_session,
        "body_ref": format!("model-surface:{body}"),
        "body_revision": "rev-1",
        "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": format!("model:{body}"),
            "inference_surface": {"contract_ref": format!("contract:{body}")}},
        "access_profile": {"schema": "actuation.instantiation/v1",
            "inference": {"allowed": ["invoke", "stream"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
        "modality_contract_ref": format!("aikit:model-modality:{body}"),
        "input_modalities": input,
        "output_modalities": output,
        "interaction": interaction,
        "transport": transport,
        "connection": connection,
        "interruption": interruption,
        "provider_binding": {"provider_ref": format!("provider:{body}"),
            "provider_session_ref": format!("provider-session:{body}"),
            "transport_connection_ref": format!("connection:{body}")},
        "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
        "resolved_at": "2026-09-17T09:00:00Z"
    }))
    .expect("fixture constitution must be valid")
}

fn context(nara: &str, session: &str) -> Value {
    json!({
        "schema": "ql.nara-dialogue-context/v1",
        "context_ref": "context:encounter-1",
        "nara_ref": nara,
        "subject_ref": "subject:frank",
        "agent_session_ref": session,
        "coordinate_ref": "M2-5-9:C0",
        "expression_ref": "expression:1",
        "expression_revision": "rev-7",
        "profile_ref": "profile:1",
        "profile_revision": "rev-2",
        "active_m_focus": "m4",
        "disclosed": [{"ref_id": "source:1", "revision": "rev-1", "standing": "verified",
            "disclosure": "khora-entry", "disclosed_via_ref": "disclosure:1"}],
        "available_action_refs": ["action:read-coordinate"],
        "expressive_act": {
            "expressive_act_ref": "expressive-act:9",
            "phase": "active",
            "basis_expression_revision": "rev-7",
            "speech_turn_ref": "response:1"
        }
    })
}

struct Actor {
    child: Child,
    input: Option<ChildStdin>,
    replies: Receiver<String>,
    sequence: usize,
}
impl Actor {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_actuation"))
            .args(["nara", "serve"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, replies) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if tx.send(line.unwrap()).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            replies,
            sequence: 0,
        }
    }
    fn send(&mut self, mut request: Value) -> Value {
        self.sequence += 1;
        request["schema"] = json!("actuation.nara-session-request/v1");
        request["request_ref"] = json!(format!("test:{}", self.sequence));
        self.raw(&(serde_json::to_string(&request).unwrap() + "\n"))
    }
    fn raw(&mut self, line: &str) -> Value {
        self.input
            .as_mut()
            .unwrap()
            .write_all(line.as_bytes())
            .unwrap();
        self.input.as_mut().unwrap().flush().unwrap();
        // A response must arrive while stdin remains OPEN: proves per-line flush.
        serde_json::from_str(
            &self
                .replies
                .recv_timeout(Duration::from_secs(10))
                .expect("actor must flush one reply per line"),
        )
        .unwrap()
    }
    fn constitute(&mut self, speech: bool, supported: bool) -> Value {
        let support = if supported {
            json!({"state":"supported"})
        } else {
            json!({"state":"unsupported","reason":"specimen without cancellation"})
        };
        self.send(json!({"operation":"constitute", "constitution":constitution("first","session:nara-1",speech,support).as_value(),
            "dialogue_context":context("nara:canonical","session:nara-1"),"allowed_action_refs":[],"denied_action_refs":[]}))
    }
    fn finish(&mut self, expected: i32) {
        self.input.take();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected));
                break;
            }
            assert!(std::time::Instant::now() < deadline, "actor did not exit");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Actor {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn native_turn_lifecycle_rejects_stale_and_duplicate_commands() {
    let mut a = Actor::start();
    assert_eq!(a.send(json!({"operation":"read"}))["ok"], false);
    assert_eq!(a.constitute(false, false)["ok"], true);
    assert_eq!(a.send(json!({"operation":"listen"}))["ok"], false);
    let active = a.send(json!({"operation":"response","response_ref":"response:1"}));
    assert_eq!(active["reading"]["speech"]["phase"], "speaking");
    assert_eq!(
        a.send(json!({"operation":"response","response_ref":"response:2"}))["ok"],
        false
    );
    assert_eq!(
        a.send(json!({"operation":"complete","response_ref":"response:stale"}))["ok"],
        false
    );
    assert_eq!(
        a.send(json!({"operation":"read"}))["reading"],
        active["reading"]
    );
    let completed = a.send(json!({"operation":"complete","response_ref":"response:1"}));
    assert_eq!(completed["reading"]["speech"]["phase"], "completed");
    let duplicate = format!(
        r#"{{"schema":"actuation.nara-session-request/v1","request_ref":"test:{}","operation":"response","response_ref":"response:2"}}"#,
        a.sequence
    );
    assert_eq!(a.raw(&(duplicate + "\n"))["ok"], false);
    assert_eq!(
        a.send(json!({"operation":"read"}))["reading"],
        completed["reading"]
    );
    let closed = a.send(json!({"operation":"close"}));
    assert_eq!(closed["actor_closed"], true);
    assert_eq!(closed["canonical_agent_session_destroyed"], false);
    a.finish(0);
}

#[test]
fn interruption_requires_attributed_effect_and_keeps_provider_claim_separate() {
    let mut a = Actor::start();
    assert_eq!(a.constitute(true, true)["ok"], true);
    let active = a.send(json!({"operation":"response","response_ref":"response:1"}));
    let mut interrupt = json!({"operation":"interrupt","interruption_ref":"interrupt:1","response_ref":"response:1",
        "reason":"user stopped output","evidence_refs":[],"at":"2026-09-27T12:00:00Z","effect":"playback-stopped"});
    assert_eq!(a.send(interrupt.clone())["ok"], false);
    assert_eq!(
        a.send(json!({"operation":"read"}))["reading"],
        active["reading"]
    );
    interrupt["evidence_refs"] = json!(["evidence:contract-validation-only"]);
    interrupt["at"] = json!("not-a-timestamp");
    assert_eq!(a.send(interrupt.clone())["ok"], false);
    interrupt["at"] = json!("2026-09-27T12:00:00Z");
    let interrupted = a.send(interrupt);
    assert_eq!(interrupted["reading"]["speech"]["phase"], "interrupted");
    assert_eq!(
        interrupted["receipt"]["interruption_receipt"]["outcome"],
        "cancelled"
    );
    assert_eq!(interrupted["transport_effect"]["provider_cancelled"], false);
    assert_eq!(
        interrupted["transport_effect"]["effect_scope"],
        "audio-playback"
    );
    a.finish(0);
}

#[test]
fn reconnect_and_context_refusals_preserve_native_binding() {
    let mut a = Actor::start();
    assert_eq!(a.constitute(true, true)["ok"], true);
    let active = a.send(json!({"operation":"response","response_ref":"response:1"}));
    let mut next = context("nara:canonical", "session:nara-1");
    next["expression_revision"] = json!("rev-8");
    let updated = a.send(json!({"operation":"context","dialogue_context":next}));
    assert_eq!(updated["reading"]["expression_revision"], "rev-8");
    assert_eq!(updated["reading"]["speech"], active["reading"]["speech"]);
    let mut reconnect = json!({"operation":"reconnect","constitution":constitution("next","session:nara-1",true,json!({"state":"supported"})).as_value(),
        "dialogue_context":context("nara:other","session:nara-1"),"change_ref":"change:1","reason":"fresh native resolution",
        "evidence_refs":["evidence:contract-validation-only"],"at":"2026-09-27T12:00:00Z"});
    assert_eq!(a.send(reconnect.clone())["ok"], false);
    assert_eq!(
        a.send(json!({"operation":"read"}))["reading"],
        updated["reading"]
    );
    reconnect["dialogue_context"] = context("nara:canonical", "session:nara-1");
    let changed = a.send(reconnect);
    assert_eq!(changed["ok"], true);
    assert_eq!(changed["reading"]["speech"]["phase"], "idle");
    assert_eq!(changed["reading"]["agent_session_ref"], "session:nara-1");
    a.finish(0);
}

#[test]
fn unsupported_interruption_does_not_change_native_turn() {
    let mut a = Actor::start();
    assert_eq!(a.constitute(false, false)["ok"], true);
    let active = a.send(json!({"operation":"response","response_ref":"response:1"}));
    let result = a.send(
        json!({"operation":"interrupt","interruption_ref":"interrupt:1",
        "response_ref":"response:1","reason":"reported transport stop",
        "evidence_refs":["evidence:contract-validation-only"],"at":"2026-09-27T12:00:00Z",
        "effect":"provider-cancelled"}),
    );
    assert_eq!(result["ok"], true);
    assert_eq!(
        result["receipt"]["interruption_receipt"]["outcome"],
        "refused"
    );
    assert_eq!(result["reading"], active["reading"]);
    assert_eq!(
        result["transport_effect"]["standing"],
        "caller-reported-native-effect"
    );
    a.finish(0);
}

#[test]
fn malformed_lines_recover_but_unbounded_or_unterminated_frames_exit() {
    let mut a = Actor::start();
    assert_eq!(a.raw("{broken\n")["ok"], false);
    assert_eq!(
        a.send(json!({"operation":"read","unexpected":true}))["ok"],
        false
    );
    assert_eq!(a.send(json!({"operation":"close"}))["ok"], true);
    a.finish(0);
    let mut a = Actor::start();
    assert_eq!(a.raw(&(" ".repeat(256 * 1024) + "\n"))["ok"], false);
    a.finish(2);
    let mut a = Actor::start();
    a.input.as_mut().unwrap().write_all(b"{}").unwrap();
    a.input.take();
    let reply: Value =
        serde_json::from_str(&a.replies.recv_timeout(Duration::from_secs(10)).unwrap()).unwrap();
    assert_eq!(reply["ok"], false);
    a.finish(2);
}
